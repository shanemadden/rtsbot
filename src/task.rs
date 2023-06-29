use serde::{Deserialize, Serialize};

use screeps::{constants::ResourceType, enums::StructureObject, game, local::ObjectId, objects::*};

use crate::{worker::WorkerReference, movement::MovementGoal};

mod build;
mod logistics;
mod repair;
mod upgrade;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TaskResult {
    Complete,
    StillWorking(Option<MovementGoal>),
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Task {
    IdleUntil(u32),
    Build(ObjectId<ConstructionSite>),
    Repair(ObjectId<Structure>),
    Upgrade(ObjectId<StructureController>),
    TakeFromResource(ObjectId<Resource>),
    TakeFromStructure(ObjectId<Structure>, ResourceType),
    DeliverToStructure(ObjectId<Structure>, ResourceType),
}

impl Task {
    pub fn run_task(&self, worker: &WorkerReference) -> TaskResult {
        match self {
            // idle creep, let's just deal with that directly
            Task::IdleUntil(tick) => {
                if game::time() >= *tick {
                    TaskResult::Complete
                } else {
                    TaskResult::StillWorking(None)
                }
            }
            // remaining task types are more complex and have handlers
            Task::Build(id) => build::build(worker, id),
            Task::Repair(id) => repair::repair(worker, id),
            Task::Upgrade(id) => upgrade::upgrade(worker, id),
            Task::TakeFromResource(id) => logistics::take_from_resource(worker, id),
            Task::TakeFromStructure(id, ty) => logistics::take_from_structure(worker, *id, *ty),
            Task::DeliverToStructure(id, ty) => logistics::deliver_to_structure(worker, *id, *ty),
        }
    }
}
