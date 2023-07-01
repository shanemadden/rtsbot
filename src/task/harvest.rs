use log::*;
use screeps::{constants::ErrorCode, local::ObjectId, objects::Source, prelude::*};

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
                    Ok(()) => TaskResult::Complete,
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            let move_goal = MovementGoal {
                                goal_pos: source.pos().into(),
                                goal_range: 1,
                                profile: MovementProfile::RoadsOneToTwo,
                                avoid_creeps: false,
                            };
                            TaskResult::StillWorking(Some(move_goal))
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
