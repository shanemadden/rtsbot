use std::collections::HashSet;
use log::*;
use serde::{Deserialize, Serialize};

use screeps::{local::RoomName, objects::Store};

use crate::{game, task::Task, worker::{Worker, WorkerRole}};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub room: RoomName,
}

impl Worker for Spawn {
    fn find_task(&self, _store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        unimplemented!()
    }

    fn can_move(&self) -> bool {
        false
    }
}
