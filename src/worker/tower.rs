use crate::{task::Task, worker::Worker};
use screeps::local::RoomName;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Tower {
    pub room: RoomName,
}

impl Worker for Tower {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
