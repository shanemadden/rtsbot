use std::collections::HashSet;
use log::*;
use serde::{Deserialize, Serialize};

use screeps::{constants::Part, local::RoomName, objects::{Store, StructureSpawn}};

use crate::{task::{Task}, worker::{Worker, WorkerRole}};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Tower {
    pub room: RoomName,
}

impl Worker for Tower {
    fn find_task(&self, _store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        unimplemented!()
    }

    fn get_body_for_creep(&self, _spawn: &StructureSpawn) -> Vec<Part> {
        panic!("can't spawn creep for tower")
    }

    fn can_move(&self) -> bool {
        false
    }
}
