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
mod tower;
mod upgrader;

pub use self::{
    builder::Builder, hauler::Hauler, source_harvester::SourceHarvester, spawn::Spawn,
    tower::Tower, upgrader::Upgrader,
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

    // when an invalid creep name is found,
    // this role is assigned so the creep can complain about it!
    Invalid(Invalid),
    // and this special case is for all spawning creeps, which will change
    // to their normal role after spawning
    SpawningCreep(SpawningCreep),
}
