use std::collections::HashSet;
use log::*;
use serde::{Deserialize, Serialize};

use screeps::{constants::Part, local::RoomName, objects::{Store, StructureSpawn}};

use crate::{game, task::{Task, TaskResult}, worker::{Worker, WorkerRole}};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Upgrader {
    #[serde(rename = "r")]
    pub home_room: RoomName,
    #[serde(rename = "i")]
    pub id: u8,
}

impl Worker for Upgrader {
    fn find_task(&self, store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        unimplemented!()
    }

    fn get_body_for_creep(&self, spawn: &StructureSpawn) -> Vec<Part> {
        unimplemented!();
    }
}
