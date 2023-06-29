use screeps::{local::ObjectId, objects::Structure};

use crate::{task::TaskResult, worker::WorkerReference};

pub fn repair(worker: &WorkerReference, target: &ObjectId<Structure>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => {
            match target.resolve() {
                Some(target_structure) => todo!(),
                // the repair target is either gone or not in a visible room;
                // a good potential enhancement here is to include the position in the repair task
                // enum, and check for visibility (moving there if not visible) before removing
                None => TaskResult::Complete,
            }
        }
        _ => panic!("unsupported worker type!"),
    }
}
