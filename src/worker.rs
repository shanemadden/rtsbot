use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use screeps::{
    constants::ResourceType,
    local::ObjectId,
    objects::{ConstructionSite, Structure, StructureController, Resource, Creep, StructureTower, StructureSpawn},
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

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Task {
    // idle until the specified tick
    IdleUntil(u8),
    Build(ObjectId<ConstructionSite>),
    Repair(ObjectId<Structure>),
    Upgrade(ObjectId<StructureController>),
    TakeFromResource(ObjectId<Resource>),
    TakeFromStructure(ObjectId<Structure>, ResourceType),
    DeliverToStructure(ObjectId<Structure>, ResourceType),
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
