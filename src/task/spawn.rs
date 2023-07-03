use log::*;
use screeps::constants::ErrorCode;

use crate::{
    role::WorkerRole,
    task::TaskResult,
    worker::{Worker, WorkerReference},
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
                    ErrorCode::Busy => TaskResult::StillWorking,
                    ErrorCode::NotEnough => TaskResult::StillWorking,
                    e => {
                        warn!("spawn failure: {:?}", e);
                        TaskResult::Complete
                    }
                },
            }
        }
        _ => panic!("unsupported worker type!"),
    }
}

pub fn wait_to_spawn(worker: &WorkerReference) -> TaskResult {
    match worker {
        WorkerReference::Creep(creep) => {
            // quick and dirty version of this is to just return working until
            // spawned, should look at creep's location for a spawn object
            // and do the math on how long it has til we spawn instead, idling
            // an appropriate length of time (and maybe setting directions last tick)
            if creep.spawning() {
                TaskResult::StillWorking
            } else {
                TaskResult::DestroyWorker
            }
        }
        _ => panic!("unsupported worker type!"),
    }
}
