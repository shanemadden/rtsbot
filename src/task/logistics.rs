use screeps::{
    constants::ResourceType,
    local::ObjectId,
    objects::{Resource, Structure},
};

use crate::{task::TaskResult, worker::WorkerReference};

pub fn take_from_resource(worker: &WorkerReference, target: &ObjectId<Resource>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(resource) => todo!(),
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}

pub fn take_from_structure(
    worker: &WorkerReference,
    target: &ObjectId<Structure>,
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

pub fn deliver_to_structure(
    worker: &WorkerReference,
    target: &ObjectId<Structure>,
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
