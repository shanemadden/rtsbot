use log::*;
use screeps::{constants::ErrorCode, local::ObjectId, objects::Structure, prelude::*};

use crate::{
    constants::*,
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn repair(
    worker: &WorkerReference,
    target: &ObjectId<Structure>,
    movement_profile: MovementProfile,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(target_structure) => match creep.repair(&target_structure) {
                Ok(()) => TaskResult::StillWorking,
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        let avoid_creeps =
                            creep.pos().get_range_to(target_structure.pos()) == RANGED_OUT_OF_RANGE;
                        let move_goal = MovementGoal {
                            pos: target_structure.pos(),
                            range: 1,
                            profile: movement_profile,
                            avoid_creeps,
                        };
                        TaskResult::MoveMeTo(move_goal)
                    }
                    e => {
                        info!("repair failure: {:?}", e);
                        TaskResult::Complete
                    }
                },
            },
            // the repair target is either gone or not in a visible room;
            // a good potential enhancement here is to include the position in the repair task
            // enum, and check for visibility (moving there if not visible) before removing
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}
