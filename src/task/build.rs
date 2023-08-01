use log::*;
use screeps::{constants::*, local::ObjectId, objects::ConstructionSite, prelude::*};

use crate::{
    constants::*,
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn build(
    worker: &WorkerReference,
    target: &ObjectId<ConstructionSite>,
    movement_profile: MovementProfile,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(construction_site) => {
                match creep.build(&construction_site) {
                    Ok(()) => TaskResult::StillWorking,
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            // if we're just out of range, we want to avoid creeps since we
                            // likely got swapped out by a crowd
                            let avoid_creeps = creep.pos().get_range_to(construction_site.pos())
                                == RANGED_OUT_OF_RANGE;
                            let move_goal = MovementGoal {
                                pos: construction_site.pos(),
                                range: 1,
                                profile: movement_profile,
                                avoid_creeps,
                            };
                            TaskResult::MoveMeTo(move_goal)
                        }
                        ErrorCode::NotEnough => TaskResult::Complete,
                        ErrorCode::InvalidTarget => {
                            // creep's standing on the construction site, and it's not walkable
                            // should maybe make it flee..
                            TaskResult::Complete
                        }
                        e => {
                            // failed for some other reason?
                            info!("build unhandled failure: {:?}", e);
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
