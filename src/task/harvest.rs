use log::*;
use screeps::{
    constants::{ErrorCode, ResourceType},
    local::ObjectId,
    objects::Source,
    prelude::*,
};

use crate::{
    constants::*,
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn harvest_energy_until_full(
    worker: &WorkerReference,
    target: &ObjectId<Source>,
    movement_profile: MovementProfile,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(source) => {
                match creep.harvest(&source) {
                    Ok(()) => {
                        let store = creep.store();
                        if store.get_free_capacity(Some(ResourceType::Energy)) == 0 {
                            TaskResult::Complete
                        } else {
                            TaskResult::StillWorking
                        }
                    }
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            let avoid_creeps =
                                creep.pos().get_range_to(source.pos()) == MELEE_OUT_OF_RANGE;
                            let move_goal = MovementGoal {
                                goal_pos: source.pos(),
                                goal_range: 1,
                                profile: movement_profile,
                                avoid_creeps,
                            };
                            TaskResult::MoveMeTo(move_goal)
                        }
                        ErrorCode::InvalidTarget => TaskResult::Complete,
                        ErrorCode::NotEnough => TaskResult::Complete,
                        e => {
                            // failed for some other reason?
                            info!("harvest failure: {:?}", e);
                            TaskResult::Complete
                        }
                    },
                }
            }
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}

pub fn harvest_energy_forever(
    worker: &WorkerReference,
    target: &ObjectId<Source>,
    movement_profile: MovementProfile,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(source) => {
                match creep.harvest(&source) {
                    Ok(()) => TaskResult::StillWorking,
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            let move_goal = MovementGoal {
                                goal_pos: source.pos(),
                                goal_range: 1,
                                profile: movement_profile,
                                avoid_creeps: false,
                            };
                            TaskResult::MoveMeTo(move_goal)
                        }
                        ErrorCode::InvalidTarget => TaskResult::Complete,
                        ErrorCode::NotEnough => TaskResult::Complete,
                        e => {
                            // failed for some other reason?
                            info!("harvest failure: {:?}", e);
                            TaskResult::Complete
                        }
                    },
                }
            }
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}
