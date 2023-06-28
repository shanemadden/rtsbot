use screeps::{
    local::ObjectId,
    objects::StructureController,
};

use crate::{worker::WorkerReference, task::TaskResult};

pub fn upgrade(worker: &WorkerReference, target: &ObjectId<StructureController>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => {
            match target.resolve() {
                Some(controller) => {
                    
                },
                None => TaskResult::Complete,
            }
        },
        _ => panic!("unsupported worker type!"),
    }
}
