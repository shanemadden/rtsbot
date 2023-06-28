use screeps::{
    local::ObjectId,
    objects::ConstructionSite,
};

use crate::{worker::WorkerReference, task::TaskResult};

pub fn build(worker: &WorkerReference, target: &ObjectId<ConstructionSite>) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => {
            match target.resolve() {
                Some(construction_site) => {

                },
                // the construction site is either gone or not in a visible room;
                // a good potential enhancement here is to include the position in the build task
                // enum, and check for visibility (moving there if not visible) before removing
                None => TaskResult::Complete,
            }
        },
        _ => panic!("unsupported worker type!"),
    }
}
