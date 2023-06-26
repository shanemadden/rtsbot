use serde::{Deserialize, Serialize};
use screeps::local::RoomName;
use crate::worker::{Task, CanFindTask};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Spawn {
    room: RoomName,
}

impl CanFindTask for Spawn {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
