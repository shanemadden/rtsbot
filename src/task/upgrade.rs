use screeps::{local::ObjectId, objects::StructureController};

use crate::{task::TaskResult, worker::WorkerReference};

pub fn upgrade(worker: &WorkerReference, target: &ObjectId<StructureController>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => match target.resolve() {
            Some(controller) => todo!(),
            None => TaskResult::Complete,
        },
        _ => panic!("unsupported worker type!"),
    }
}
