use std::collections::HashMap;

use log::*;
use screeps::{game, RoomName};
use wasm_bindgen::prelude::*;

mod logging;
mod movement;
mod worker;

use crate::worker::{WorkerId, WorkerRole, WorkerState};

// add wasm_bindgen to any function you would like to expose for call from js
#[wasm_bindgen]
pub fn setup() {
    // show all output of Info level, adjust as needed
    logging::setup_logging(logging::Info);
}

// this is one method of persisting data on the wasm memory heap between ticks
// this is an alternative to keeping state in memory on game objects - but will be lost on
// global resets, which occur at differing frequencies on different server environments
static mut SHARD_STATE: Option<ShardState> = None;

// define the giant struct which holds all of state data we're interested in holding
// for future ticks
pub struct ShardState {
    // the tick when this state was created
    pub global_init_time: u32,
    // workers and their task queues (includes creeps as well as structures)
    pub worker_state: HashMap<(worker::WorkerId, worker::WorkerRole), worker::WorkerState>,
    // owned room states and spawn queues
    pub colony_state: HashMap<RoomName, ColonyState>,
}

impl Default for ShardState {
    fn default() -> ShardState {
        ShardState {
            global_init_time: game::time(),
            worker_state: HashMap::new(),
            colony_state: HashMap::new(),
        }
    }
}

pub struct ColonyState {
    // todo add stuff here - spawn queue, maybe remote tracking
}

// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    debug!("loop starting! CPU: {}", game::cpu::get_used());
    let tick = game::time();

    // SAFETY: only one instance of the game loop can be running at a time
    // We must use this same mutable reference throughout the entire tick,
    // as any other access to it would cause undefined behavior!
    let mut shard_state = unsafe { SHARD_STATE.get_or_insert_with(|| ShardState::default()) };

    // register all creeps that aren't yet in our tracking, and delete the state of any that we can no longer see
    worker::scan_and_register_creeps(&mut shard_state);

    // scan for new worker structures as well - every 100 ticks, or if this is the startup tick
    if tick % 100 == 0 || tick == shard_state.global_init_time {
        worker::scan_and_register_structures(&mut shard_state);
    }

    // run all registered workers, deleting those that we can't resolve
    worker::run_workers(&mut shard_state);

    // run movement phase now that all workers have run, deleting the references to game objects from the current
    // tick (so that we can ensure they aren't used in future ticks as well as to enable them to be GC'd in js)
    movement::run_movement_and_remove_references(&mut shard_state);

    info!("done! cpu: {}, global age {}", game::cpu::get_used(), game::time() - shard_state.global_init_time)
}
