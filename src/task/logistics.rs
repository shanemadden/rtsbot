use log::*;
use screeps::{
    constants::{ErrorCode, ResourceType},
    enums::StructureObject,
    local::ObjectId,
    objects::{Resource, Structure},
    prelude::*,
};

use crate::{
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::WorkerReference,
};

pub fn take_from_resource(
    worker: &WorkerReference,
    target: &ObjectId<Resource>,
    _movement_profile: MovementProfile,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(resource) => {
                match creep.pickup(&resource) {
                    Ok(()) => TaskResult::Complete,
                    Err(e) => match e {
                        ErrorCode::NotInRange => {
                            let move_goal = MovementGoal {
                                pos: resource.pos(),
                                range: 1,
                                // store is empty, no fatigue from carry parts - override with 5:1
                                profile: MovementProfile::SwampFiveToOne,
                                avoid_creeps: false,
                            };
                            TaskResult::MoveMeTo(move_goal)
                        }
                        ErrorCode::InvalidTarget => TaskResult::Complete,
                        ErrorCode::NotEnough => TaskResult::Complete,
                        ErrorCode::Full => TaskResult::Complete,
                        e => {
                            // failed for some other reason?
                            warn!("pickup unhandled failure: {:?}", e);
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

pub fn take_from_structure(
    worker: &WorkerReference,
    target: ObjectId<Structure>,
    resource_type: ResourceType,
    _movement_profile: MovementProfile,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(structure) => {
                let structure_object = StructureObject::from(structure);
                match structure_object.as_withdrawable() {
                    Some(withdrawable) => match creep.withdraw(withdrawable, resource_type, None) {
                        Ok(()) => TaskResult::Complete,
                        Err(e) => match e {
                            ErrorCode::NotInRange => {
                                let move_goal = MovementGoal {
                                    pos: structure_object.pos(),
                                    range: 1,
                                    // store is empty, no fatigue from carry parts - override with 5:1
                                    profile: MovementProfile::SwampFiveToOne,
                                    avoid_creeps: false,
                                };
                                TaskResult::MoveMeTo(move_goal)
                            }
                            ErrorCode::InvalidTarget => TaskResult::Complete,
                            ErrorCode::NotEnough => TaskResult::Complete,
                            ErrorCode::Full => TaskResult::Complete,
                            e => {
                                // failed for some other reason?
                                warn!("withdraw unhandled failure: {:?}", e);
                                TaskResult::Complete
                            }
                        },
                    },
                    None => {
                        warn!("withdraw attempted from structure without store?");
                        TaskResult::Complete
                    }
                }
            }
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}

pub fn deliver_to_structure(
    worker: &WorkerReference,
    target: ObjectId<Structure>,
    resource_type: ResourceType,
    movement_profile: MovementProfile,
) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(structure) => {
                let structure_object = StructureObject::from(structure);
                match structure_object.as_transferable() {
                    Some(transferable) => match creep.transfer(transferable, resource_type, None) {
                        Ok(()) => TaskResult::Complete,
                        Err(e) => match e {
                            ErrorCode::NotInRange => {
                                let move_goal = MovementGoal {
                                    pos: structure_object.pos(),
                                    range: 1,
                                    profile: movement_profile,
                                    avoid_creeps: false,
                                };
                                TaskResult::MoveMeTo(move_goal)
                            }
                            ErrorCode::InvalidTarget => TaskResult::Complete,
                            ErrorCode::NotEnough => TaskResult::Complete,
                            ErrorCode::Full => TaskResult::Complete,
                            e => {
                                // failed for some other reason?
                                warn!("transfer unhandled failure: {:?}", e);
                                TaskResult::Complete
                            }
                        },
                    },
                    None => {
                        warn!("transfer attempted to structure without store?");
                        TaskResult::Complete
                    }
                }
            }
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}
