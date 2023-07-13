use serde::{Deserialize, Serialize};

use screeps::{
    constants::ResourceType,
    game,
    local::{ObjectId, Position},
    objects::*,
};

use crate::{
    movement::{MovementGoal, MovementProfile},
    role::WorkerRole,
    worker::WorkerReference,
};

mod build;
mod harvest;
mod logistics;
mod repair;
mod spawn;
mod upgrade;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TaskResult {
    Complete,
    StillWorking,
    MoveMeTo(MovementGoal),
    AddTaskToFront(Task),
    CompleteAddTaskToFront(Task),
    CompleteAddTaskToBack(Task),
    DestroyWorker,
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Task {
    IdleUntil(u32),
    MoveToPosition(Position, u32),
    HarvestEnergyUntilFull(ObjectId<Source>),
    HarvestEnergyForever(ObjectId<Source>),
    Build(ObjectId<ConstructionSite>),
    Repair(ObjectId<Structure>),
    Upgrade(ObjectId<StructureController>),
    TakeFromResource(ObjectId<Resource>),
    TakeFromStructure(ObjectId<Structure>, ResourceType),
    DeliverToStructure(ObjectId<Structure>, ResourceType),
    SpawnCreep(WorkerRole),
    WaitToSpawn,
}

impl Task {
    pub fn run_task(
        &self,
        worker: &WorkerReference,
        movement_profile: MovementProfile,
    ) -> TaskResult {
        match self {
            // idle worker, let's just deal with that directly
            Task::IdleUntil(tick) => {
                if game::time() >= *tick {
                    TaskResult::Complete
                } else {
                    TaskResult::StillWorking
                }
            }
            Task::MoveToPosition(position, range) => {
                if worker.pos().get_range_to(*position) <= *range {
                    TaskResult::Complete
                } else {
                    TaskResult::MoveMeTo(MovementGoal {
                        pos: *position,
                        range: *range,
                        profile: movement_profile,
                        avoid_creeps: false,
                    })
                }
            }
            // remaining task types are more complex and have handlers
            Task::HarvestEnergyUntilFull(id) => {
                harvest::harvest_energy_until_full(worker, id, movement_profile)
            }
            Task::HarvestEnergyForever(id) => {
                harvest::harvest_energy_forever(worker, id, movement_profile)
            }
            Task::Build(id) => build::build(worker, id, movement_profile),
            Task::Repair(id) => repair::repair(worker, id, movement_profile),
            Task::Upgrade(id) => upgrade::upgrade(worker, id, movement_profile),
            Task::TakeFromResource(id) => {
                logistics::take_from_resource(worker, id, movement_profile)
            }
            Task::TakeFromStructure(id, ty) => {
                logistics::take_from_structure(worker, *id, *ty, movement_profile)
            }
            Task::DeliverToStructure(id, ty) => {
                logistics::deliver_to_structure(worker, *id, *ty, movement_profile)
            }
            Task::SpawnCreep(role) => spawn::spawn_creep(worker, role),
            Task::WaitToSpawn => spawn::wait_to_spawn(worker),
        }
    }
}
