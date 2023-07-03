use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use screeps::{
    constants::Part,
    local::RoomName,
    objects::{Store, StructureSpawn},
};

use crate::{role::WorkerRole, task::Task, worker::Worker};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Upgrader {
    #[serde(rename = "r")]
    pub home_room: RoomName,
    #[serde(rename = "i")]
    pub id: u8,
}

impl Worker for Upgrader {
    fn find_task(&self, _store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        unimplemented!()
    }

    fn get_body_for_creep(&self, _spawn: &StructureSpawn) -> Vec<Part> {
        unimplemented!();
    }
}
