use crate::worker::{CanFindTask, Task};
use screeps::local::RoomName;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Tower {
    pub room: RoomName,
}

impl CanFindTask for Tower {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
