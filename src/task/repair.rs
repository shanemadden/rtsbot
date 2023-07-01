use log::*;
use screeps::{constants::ErrorCode, local::ObjectId, objects::Structure};

use crate::{
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn repair(worker: &WorkerReference, target: &ObjectId<Structure>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(target_structure) => match creep.repair(&target_structure) {
                Ok(()) => TaskResult::StillWorking(None),
                Err(e) => match e {
                    ErrorCode::NotInRange => {
                        let move_goal = MovementGoal {
                            goal_pos: target_structure.pos().into(),
                            goal_range: 1,
                            profile: MovementProfile::RoadsOneToTwo,
                            avoid_creeps: false,
                        };
                        TaskResult::StillWorking(Some(move_goal))
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
