use log::*;
use screeps::{
    constants::{ResourceType, ErrorCode},
    enums::StructureObject,
    local::ObjectId,
    objects::{Structure, Resource},
    prelude::*,
};

use crate::{task::TaskResult, worker::WorkerReference, movement::{MovementGoal, MovementProfile}};

pub fn take_from_resource(worker: &WorkerReference, target: &ObjectId<Resource>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(resource) =>  {
                match creep.pickup(&resource) {
                    Ok(()) => TaskResult::Complete,
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            let move_goal = MovementGoal {
                                goal: resource.pos().into(),
                                goal_range: 1,
                                priority: 1,
                                profile: MovementProfile::RoadsOneToTwo,
                                avoid_creeps: false,
                            };
                            TaskResult::StillWorking(Some(move_goal))
                        },
                        ErrorCode::InvalidTarget => TaskResult::Complete,
                        ErrorCode::NotEnough => TaskResult::Complete,
                        ErrorCode::Full => TaskResult::Complete,
                        e => {
                            // failed for some other reason?
                            info!("build failure: {:?}", e);
                            TaskResult::Complete
                        }
                    }
                }
            },
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}

pub fn take_from_structure(
    worker: &WorkerReference,
    target: ObjectId<Structure>,
    resource_type: ResourceType,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(structure) =>  {
                let structure_object = StructureObject::from(structure);
                match structure_object.as_withdrawable() {
                    Some(has_store) => match creep.withdraw(has_store, resource_type, None) {
                        Ok(()) => TaskResult::Complete,
                        Err(e) => match e {
                            ErrorCode::NotInRange => {
                                let move_goal = MovementGoal {
                                    goal: structure_object.pos().into(),
                                    goal_range: 1,
                                    priority: 1,
                                    profile: MovementProfile::RoadsOneToTwo,
                                    avoid_creeps: false,
                                };
                                TaskResult::StillWorking(Some(move_goal))
                            },
                            ErrorCode::InvalidTarget => TaskResult::Complete,
                            ErrorCode::NotEnough => TaskResult::Complete,
                            ErrorCode::Full => TaskResult::Complete,
                            e => {
                                // failed for some other reason?
                                info!("build failure: {:?}", e);
                                TaskResult::Complete
                            }
                        }
                    },
                    None => {
                        // failed for some other reason?
                        info!("withdraw attempted from structure without store?");
                        TaskResult::Complete
                    }
                }
            },
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}

pub fn deliver_to_structure(
    worker: &WorkerReference,
    target: ObjectId<Structure>,
    resource_type: ResourceType,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(structure) => todo!(),
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}
