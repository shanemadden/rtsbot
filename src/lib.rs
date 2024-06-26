use js_sys::JsString;
use log::*;
use screeps::{
    game,
    local::{Position, RawObjectId, RoomCoordinate, RoomName},
};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

mod logging;
mod movement;
mod role;
mod task;
mod worker;

use self::{
    role::WorkerRole,
    task::Task,
    worker::{WorkerId, WorkerState},
};

/// Tunable important numbers for the bot, in one place for convenience
mod constants {
    use screeps::constants::{Part::*, *};
    /// Won't do pathing for moving creeps if current-tick CPU spend is above this level when movement step is reached
    pub const HIGH_CPU_THRESHOLD: f64 = 250.;
    /// Won't do pathing for moving creeps if bucket is below this number
    pub const LOW_BUCKET_THRESHOLD: i32 = 1_000;
    /// Consider creeps to be stuck and get them a new path after this many ticks
    pub const STUCK_REPATH_THRESHOLD: u8 = 10;
    /// Limit for pathfinder ops
    pub const MAX_OPS: u32 = 100_000;
    /// Limit for pathfinder rooms
    pub const MAX_ROOMS: u8 = 64;
    /// A* heuristic weight - default is 1.2, but it risks non-optimal paths, so we turn it down a bit
    pub const HEURISTIC_WEIGHT: f64 = 1.0;
    /// When task finding fails, idle this long
    pub const NO_TASK_IDLE_TICKS: u32 = 10;
    /// Builder role considers energy on the groundfor grabbing above this amount
    pub const BUILDER_ENERGY_PICKUP_THRESHOLD: u32 = 100;
    /// Builder role considers energy for withdraw from structures above this amount
    pub const BUILDER_ENERGY_WITHDRAW_THRESHOLD: u32 = 1_000;
    /// Cost of each set of hauler body parts (2 carry, 1 move)
    pub const HAULER_COST_PER_MULTIPLIER: u32 = Carry.cost() * 2 + Move.cost();
    /// Count of parts in the hauler body set
    pub const HAULER_PARTS_PER_MULTIPLIER: u32 = 3;
    /// How many haulers to try to keep alive in each room
    pub const HAULER_COUNT_TARGET: u8 = 1;
    /// Largest number of sets to allow a hauler to be spawned with
    pub const HAULER_MAX_MULTIPLIER: u32 = MAX_CREEP_SIZE / HAULER_PARTS_PER_MULTIPLIER;
    /// Hauler role considers energy on the ground for grabbing above this amount
    pub const HAULER_ENERGY_PICKUP_THRESHOLD: u32 = 35;
    /// Hauler role considers energy for withdraw from structures above this amount
    pub const HAULER_ENERGY_WITHDRAW_THRESHOLD: u32 = 500;
    /// Builder role repair maximum at RCL1
    pub const REPAIR_WATERMARK_RCL_1: u32 = 10_000;
    /// Builder role repair maximum at RCL2
    pub const REPAIR_WATERMARK_RCL_2: u32 = 10_000;
    /// Builder role repair maximum at RCL3
    pub const REPAIR_WATERMARK_RCL_3: u32 = 50_000;
    /// Builder role repair maximum at RCL4
    pub const REPAIR_WATERMARK_RCL_4: u32 = 100_000;
    /// Builder role repair maximum at RCL5
    pub const REPAIR_WATERMARK_RCL_5: u32 = 100_000;
    /// Builder role repair maximum at RCL6
    pub const REPAIR_WATERMARK_RCL_6: u32 = 500_000;
    /// Builder role repair maximum at RCL7
    pub const REPAIR_WATERMARK_RCL_7: u32 = 1_000_000;
    /// Builder role repair maximum at RCL8
    pub const REPAIR_WATERMARK_RCL_8: u32 = 3_000_000;
    /// How many do-it-all creeps to keep alive at RCL 1
    pub const STARTUP_RCL1_COUNT_TARGET: u8 = 15;
    /// How many upgraders to try to keep alive in each room
    pub const UPGRADER_COUNT_TARGET: u8 = 4;
    /// Builder role considers energy on the groundfor grabbing above this amount
    pub const UPGRADER_ENERGY_PICKUP_THRESHOLD: u32 = 100;
    /// Upgrader roler considers energy for withdraw from structures above this amount
    pub const UPGRADER_ENERGY_WITHDRAW_THRESHOLD: u32 = 1_200;
    /// Fill terminals to this much energy
    pub const TERMINAL_ENERGY_TARGET: u32 = 50_000;
    /// Creeps are just out of range of their ranged action at this range; at this range
    /// they'll usually path avoiding creeps
    pub const RANGED_OUT_OF_RANGE: u32 = (CREEP_RANGED_ACTION_RANGE + 1) as u32;
    /// Creeps are just out of range of their melee action at this range; at this range
    /// they'll usually path avoiding creeps
    pub const MELEE_OUT_OF_RANGE: u32 = 2;
}

// this is one method of persisting data on the wasm memory heap between ticks
// this is an alternative to keeping state in memory on game objects - but will be lost on
// global resets, which occur at differing frequencies on different server environments
static mut SHARD_STATE: Option<ShardState> = None;
static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

// define the giant struct which holds all of state data we're interested in holding
// for future ticks
pub struct ShardState {
    // the tick when this state was created
    pub global_init_time: u32,
    // owned room states and spawn queues
    pub colony_state: HashMap<RoomName, ColonyState>,
    // workers and their task queues (includes creeps as well as structures)
    pub worker_state: HashMap<WorkerId, WorkerState>,
    // additionally, a HashSet<WorkerRole> where we'll mark which roles
    // we have active workers for, allowing spawns to check which workers to create
    pub worker_roles: HashSet<WorkerRole>,
}

impl Default for ShardState {
    fn default() -> ShardState {
        ShardState {
            global_init_time: game::time(),
            colony_state: HashMap::new(),
            worker_state: HashMap::new(),
            worker_roles: HashSet::new(),
        }
    }
}

pub struct ColonyState {
    // todo add stuff here - spawn queue, maybe remote tracking
}

#[wasm_bindgen]
pub fn update_selected_object(client_tick: u32, object_id: JsString, object_type: String) {
    let raw_obj: RawObjectId = object_id.try_into().unwrap();
    info!(
        "selection updated! {} {:?} {}",
        client_tick, raw_obj, object_type
    );
}

#[wasm_bindgen]
pub fn right_click_position(
    room_name: JsString,
    x: u8,
    y: u8,
    object_id: JsString,
    object_type: String,
) {
    let pos = Position::new(
        RoomCoordinate::try_from(x).unwrap(),
        RoomCoordinate::try_from(y).unwrap(),
        RoomName::try_from(room_name).unwrap(),
    );
    info!("click observed: {}, {} {}", pos, object_id, object_type);

    if object_type == "creep" {
        let shard_state = unsafe { SHARD_STATE.get_or_insert_with(ShardState::default) };
        let id_raw: RawObjectId = object_id.try_into().unwrap();
        shard_state
            .worker_state
            .entry(WorkerId::Creep(id_raw.into()))
            .and_modify(|state| state.task_queue.push_front(Task::MoveToPosition(pos, 0)));
    }
}

#[wasm_bindgen]
pub fn wasm_loop() {
    INIT_LOGGING.call_once(|| {
        // show all output of Info level, adjust as needed
        logging::setup_logging(logging::Info);
    });

    let tick = game::time();
    info!("tick {} starting! CPU: {:.4}", tick, game::cpu::get_used());

    // SAFETY: only one instance of the game loop can be running at a time
    // We must use this same mutable reference throughout the entire tick,
    // as any other access to it would cause undefined behavior!
    let shard_state = unsafe { SHARD_STATE.get_or_insert_with(ShardState::default) };

    // register all creeps that aren't yet in our tracking, and delete the state of any that we can
    // no longer see
    worker::scan_and_register_creeps(shard_state);

    // scan for new worker structures as well - every 100 ticks, or if this is the startup tick
    if tick % 100 == 0 || tick == shard_state.global_init_time {
        worker::scan_and_register_structures(shard_state);
    }

    // run all registered workers, attempting to resolve those that haven't already and deleting
    // any workers that don't resolve

    // game state changes like spawning creeps will start happening here, so this is
    // intentionally ordered after we've completed all worker scanning for the tick so we
    // don't need to think about the case of dealing with the object stubs of creeps whose
    // spawn started this tick
    worker::run_workers(shard_state);

    // run movement phase now that all workers have run, while deleting the references to game
    // objects from the current tick (as a way to ensure they aren't used in future ticks
    // as well as to enable them to be GC'd and their memory freed in js heap, if js wants to)
    movement::run_movement_and_remove_worker_refs(shard_state);

    info!(
        "tick {} done! cpu: {:.4}, execution instance age {}",
        tick,
        game::cpu::get_used(),
        tick - shard_state.global_init_time
    )
}
