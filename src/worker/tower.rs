use std::collections::HashSet;
use log::*;
use serde::{Deserialize, Serialize};

use screeps::{local::RoomName, objects::Store};

use crate::{game, task::Task, worker::{Worker, WorkerRole}};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Tower {
    pub room: RoomName,
}

impl Worker for Tower {
    fn find_task(&self, store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        unimplemented!()
    }

    fn can_move(&self) -> bool {
        false
    }
}
