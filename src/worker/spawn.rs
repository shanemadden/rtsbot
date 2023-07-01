use crate::{task::Task, worker::Worker};
use screeps::local::RoomName;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub room: RoomName,
}

impl Worker for Spawn {
    fn find_task(&self) -> Task {
        unimplemented!()
    }

    fn can_move(&self) -> bool {
        false
    }
}
