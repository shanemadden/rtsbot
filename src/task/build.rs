use log::*;
use screeps::{constants::ErrorCode, local::ObjectId, objects::ConstructionSite};

use crate::{
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn build(worker: &WorkerReference, target: &ObjectId<ConstructionSite>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(construction_site) => {
                match creep.build(&construction_site) {
                    Ok(()) => TaskResult::StillWorking(None),
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            let move_goal = MovementGoal {
                                goal: construction_site.pos().into(),
                                goal_range: 1,
                                priority: 1,
                                profile: MovementProfile::RoadsOneToTwo,
                                avoid_creeps: false,
                            };
                            TaskResult::StillWorking(Some(move_goal))
                        }
                        ErrorCode::InvalidTarget => {
                            // creep's standing on the construction site, and it's not walkable
                            // should maybe make it flee..
                            TaskResult::Complete
                        }
                        e => {
                            // failed for some other reason?
                            info!("build failure: {:?}", e);
                            TaskResult::Complete
                        }
                    },
                }
            }
            // the construction site is either gone or not in a visible room;
            // a good potential enhancement here is to include the position in the build task
            // enum, and check for visibility (moving there if not visible) before removing
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}
