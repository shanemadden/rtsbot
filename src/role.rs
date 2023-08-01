use std::collections::HashSet;

use enum_dispatch::enum_dispatch;

use serde::{Deserialize, Serialize};

use screeps::{
    constants::Part,
    objects::{Store, StructureSpawn},
};

use crate::{task::Task, worker::Worker};

mod builder;
mod hauler;
mod source_harvester;
mod spawn;
mod startup;
mod tower;
mod upgrader;

pub use self::{
    builder::Builder,
    hauler::Hauler,
    source_harvester::SourceHarvester,
    spawn::Spawn,
    startup::Startup,
    tower::Tower,
    upgrader::Upgrader,
};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Invalid {}

impl Worker for Invalid {
    fn find_task(&self, _store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        // broken creep, name didn't parse! doom creep to idle until the end of time
        Task::IdleUntil(u32::MAX)
    }

    fn get_body_for_creep(&self, _spawn: &StructureSpawn) -> Vec<Part> {
        panic!("can't spawn invalid workers!")
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SpawningCreep {}

impl Worker for SpawningCreep {
    fn find_task(&self, _store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        Task::WaitToSpawn
    }

    fn get_body_for_creep(&self, _spawn: &StructureSpawn) -> Vec<Part> {
        panic!("can't spawn spawning workers!")
    }
}

/// represents all types of worker role, along with
/// any embedded data needed for the worker to rebuild its state;
/// the serialized version of these are used for creep names
#[enum_dispatch(Worker)]
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WorkerRole {
    #[serde(rename = "🛠")]
    Builder(Builder),
    #[serde(rename = "🧙")]
    Upgrader(Upgrader),
    #[serde(rename = "🐿")]
    Hauler(Hauler),
    #[serde(rename = "⛏")]
    SourceHarvester(SourceHarvester),
    #[serde(rename = "🐜")]
    Startup(Startup),

    // structures
    Spawn(Spawn),
    Tower(Tower),

    // creeps with unparseable names
    Invalid(Invalid),
    // spawning creeps
    SpawningCreep(SpawningCreep),
}
