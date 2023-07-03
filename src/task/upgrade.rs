use log::*;
use screeps::{constants::ErrorCode, local::ObjectId, objects::StructureController};

use crate::{
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn upgrade(worker: &WorkerReference, target: &ObjectId<StructureController>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(controller) => match creep.upgrade_controller(&controller) {
                Ok(()) => TaskResult::StillWorking,
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        let move_goal = MovementGoal {
                            goal_pos: controller.pos().into(),
                            goal_range: 1,
                            profile: MovementProfile::RoadsOneToTwo,
                            avoid_creeps: false,
                        };
                        TaskResult::MoveMeTo(move_goal)
                    }
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
