use crate::worker::{CanFindTask, Task};
use screeps::local::RoomName;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub room: RoomName,
}

impl CanFindTask for Spawn {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
