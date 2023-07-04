use log::*;
use screeps::{constants::ErrorCode, game, local::ObjectId, objects::Source, prelude::*};

use crate::{
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn harvest_energy(worker: &WorkerReference, target: &ObjectId<Source>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(source) => {
                match creep.harvest(&source) {
                    Ok(()) => {
                        // harvest tasks don't fail when we're full, but we don't want builder
                        // creeps that get harvest tasks to sit there dumping energy on the floor
                        // bail from the task every 10 ticks; dedicated harvests will find it again
                        // todo nah this sucks we need to look at the store
                        if game::time() % 100 == 0 {
                            TaskResult::Complete
                        } else {
                            TaskResult::StillWorking
                        }
                    }
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            let move_goal = MovementGoal {
                                goal_pos: source.pos(),
                                goal_range: 1,
                                profile: MovementProfile::RoadsOneToTwo,
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
