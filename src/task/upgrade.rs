use log::*;
use screeps::{constants::ErrorCode, local::ObjectId, objects::StructureController, prelude::*};

use crate::{
    constants::*,
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn upgrade(
    worker: &WorkerReference,
    target: &ObjectId<StructureController>,
    movement_profile: MovementProfile,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(controller) => match creep.upgrade_controller(&controller) {
                Ok(()) => TaskResult::StillWorking,
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        let avoid_creeps =
                            creep.pos().get_range_to(controller.pos()) == RANGED_OUT_OF_RANGE;
                        let move_goal = MovementGoal {
                            pos: controller.pos(),
                            range: 1,
                            profile: movement_profile,
                            avoid_creeps,
                        };
                        TaskResult::MoveMeTo(move_goal)
                    }
                    ErrorCode::NotEnough => TaskResult::Complete,
                    e => {
                        info!("upgrade failure: {:?}", e);
                        TaskResult::Complete
                    }
                },
            },
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}
