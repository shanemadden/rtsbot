use log::*;
use screeps::{constants::ErrorCode, local::ObjectId, objects::StructureController};

use crate::{
    movement::{MovementGoal, MovementProfile},
    task::TaskResult,
    worker::{WorkerReference, WorkerRole, Worker},
};

pub fn spawn_creep(worker: &WorkerReference, role: &WorkerRole) -> TaskResult {
    match worker {
        WorkerReference::Spawn(spawn) => {
            // serialize the name here and pass it through
            let name = serde_json::to_string(&role).expect("roles should all serialize");
            let body = role.get_body_for_creep(&spawn);
            match spawn.spawn_creep(&body, &name) {
                Ok(()) => TaskResult::Complete,
                Err(e) => match e {
                    // already have a creep with this name
                    ErrorCode::NameExists => TaskResult::Complete,
                    ErrorCode::Busy => TaskResult::StillWorking(None),
                    ErrorCode::NotEnough => TaskResult::StillWorking(None),
                    e => {
                        warn!("spawn failure: {:?}", e);
                        TaskResult::Complete
                    }
                },
            }
        },
        _ => panic!("unsupported worker type!"),
    }
}
