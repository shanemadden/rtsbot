use std::collections::VecDeque;

use enum_dispatch::enum_dispatch;
use log::*;
use serde::{Deserialize, Serialize};

use screeps::{
    constants::find,
    enums::StructureObject,
    game,
    local::ObjectId,
    objects::{Creep, StructureSpawn, StructureTower},
    prelude::*,
};

use crate::{
    movement::{MovementGoal, PathState},
    task::Task,
    ShardState,
};

mod builder;
mod hauler;
mod source_harvester;
mod spawn;
mod tower;
mod upgrader;

use self::builder::Builder;
use self::hauler::Hauler;
use self::source_harvester::SourceHarvester;
use self::spawn::Spawn;
use self::tower::Tower;
use self::upgrader::Upgrader;

// an enum to represent all of the different types of 'worker' object id we may have
// for resolving the objects each tick for work
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone)]
pub enum WorkerId {
    Creep(ObjectId<Creep>),
    Spawn(ObjectId<StructureSpawn>),
    Tower(ObjectId<StructureTower>),
}

// resolve the actual worker object if it still exists
impl WorkerId {
    pub fn resolve(&self) -> Option<WorkerReference> {
        match self {
            WorkerId::Creep(id) => id.resolve().map(|o| WorkerReference::Creep(o)),
            WorkerId::Spawn(id) => id.resolve().map(|o| WorkerReference::Spawn(o)),
            WorkerId::Tower(id) => id.resolve().map(|o| WorkerReference::Tower(o)),
        }
    }
}

// an enum to represent all of the different types of 'worker' object, in resolved form
// to avoid using stale up
#[derive(Debug, Clone)]
pub enum WorkerReference {
    Creep(Creep),
    Spawn(StructureSpawn),
    Tower(StructureTower),
}

// trait to declare the functions that each role needs to implement
// to be called by enum_dispatch
#[enum_dispatch]
pub trait CanFindTask {
    // function to be called for the worker when it has no work to do,
    // so that it can find another task (even if it's just to idle)
    fn find_task(&self) -> Task;
}

// an enum to represent all types of worker role, along with
// any embedded data needed for the worker to rebuild its state;
// the serialized version of these are used for creep names
#[enum_dispatch(CanFindTask)]
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WorkerRole {
    // Creep worker types, with assigned room name
    // and a unique number for when there might be multiple
    #[serde(rename = "🛠")]
    Builder(Builder),
    #[serde(rename = "🧙")]
    Upgrader(Upgrader),
    #[serde(rename = "🐿")]
    Hauler(Hauler),
    // static harvester that gets a pre-assigned source for life,
    // so we just give it a position of the destination source
    #[serde(rename = "⛏")]
    SourceHarvester(SourceHarvester),

    // structures with worker roles
    Spawn(Spawn),
    Tower(Tower),
}

#[derive(Debug, Clone)]
pub struct WorkerState {
    pub role: WorkerRole,
    pub task_queue: VecDeque<Task>,
    pub worker_reference: Option<WorkerReference>,
    pub movement_goal: Option<MovementGoal>,
    pub path_state: Option<PathState>,
}

impl WorkerState {
    pub fn new_with_role_and_reference(
        role: WorkerRole,
        worker_reference: WorkerReference,
    ) -> WorkerState {
        WorkerState {
            role,
            task_queue: VecDeque::new(),
            worker_reference: Some(worker_reference),
            movement_goal: None,
            path_state: None,
        }
    }
}

pub fn scan_and_register_creeps(shard_state: &mut ShardState) {
    for creep in game::creeps().values() {
        if creep.spawning() {
            // we don't want to work with spawning creeps, skip this one!
            continue;
        }

        // this function is called at the start of tick before any tasks, so we can simply assume
        // every creep has an id; if spawning had run then id-free creeps would be a possibility.
        let id = WorkerId::Creep(creep.try_id().expect("expected creep to have id!"));

        // update the reference if there's already a worker for this creep id,
        // or parse the name and add it if it's not there
        shard_state
            .worker_state
            .entry(id)
            .and_modify(|worker_state| {
                worker_state.worker_reference = Some(WorkerReference::Creep(creep))
            })
            .or_insert_with(|| {
                let creep_name = creep.name();
                match serde_json::from_str(&creep_name) {
                    Ok(role) => WorkerState::new_with_role_and_reference(
                        role,
                        WorkerReference::Creep(creep),
                    ),
                    Err(e) => {
                        // creep name couldn't parse! but, we're in or_insert_with, so we're
                        // expected to insert something; killing the creep and crashing is
                        // brittle but gets us to where the creeps are all valid, eventually! ;)
                        let _ = creep.suicide();
                        panic!("couldn't parse creep name {}: {:?}", creep_name, e);
                    }
                }
            });
    }
}

pub fn scan_and_register_structures(shard_state: &mut ShardState) {
    for room in game::rooms().values() {
        // narrowing the scan down to just rooms that are owned currently,
        // as all structure types that are 'workers' in this bot can only
        // function in owned rooms
        let owned = room
            .controller()
            .map_or(false, |controller| controller.my());

        if owned {
            let room_name = room.name();

            for structure in room.find(find::MY_STRUCTURES, None) {
                match structure {
                    StructureObject::StructureSpawn(spawn) => {
                        let id = WorkerId::Spawn(spawn.id());
                        let role = WorkerRole::from(Spawn { room: room_name });
                        let worker_state = WorkerState::new_with_role_and_reference(
                            role,
                            WorkerReference::Spawn(spawn),
                        );
                        shard_state.worker_state.insert(id, worker_state);
                    }
                    StructureObject::StructureTower(tower) => {
                        let id = WorkerId::Tower(tower.id());
                        let role = WorkerRole::Tower(Tower { room: room_name });
                        let worker_state = WorkerState::new_with_role_and_reference(
                            role,
                            WorkerReference::Tower(tower),
                        );
                        shard_state.worker_state.insert(id, worker_state);
                    }
                    // we don't make workers for any other structure types!
                    _ => {}
                }
            }
        }
    }
}

pub fn run_workers(shard_state: &mut ShardState) {
    unimplemented!()
}
