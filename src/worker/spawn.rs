use log::*;
use serde::{Deserialize, Serialize};

use screeps::{local::RoomName, objects::Store};

use crate::{game, task::Task, worker::Worker};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub room: RoomName,
}

impl Worker for Spawn {
    fn find_task(&self, store: &Store) -> Task {
        unimplemented!()
    }

    fn can_move(&self) -> bool {
        false
    }
}
