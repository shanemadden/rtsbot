use log::*;
use screeps::{
    constants::ErrorCode, enums::StructureObject, local::ObjectId, objects::Structure, prelude::*,
};

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
            Some(target_structure) => {
                let structure_object = StructureObject::from(target_structure);
                match structure_object.as_repairable() {
                    Some(repairable) => match creep.repair(repairable) {
                        Ok(()) => TaskResult::StillWorking,
                        Err(e) => match e {
                            ErrorCode::NotInRange => {
                                let avoid_creeps = creep.pos().get_range_to(structure_object.pos())
                                    == RANGED_OUT_OF_RANGE;
                                let move_goal = MovementGoal {
                                    pos: structure_object.pos(),
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
                    // repair target isn't repairable? oh well!
                    None => TaskResult::Complete,
                }
            }
            // the repair target is either gone or not in a visible room;
            // a good potential enhancement here is to include the position in the repair task
            // enum, and check for visibility (moving there if not visible) before removing
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}
