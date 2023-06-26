use serde::{Deserialize, Serialize};
use screeps::local::RoomName;
use crate::worker::{Task, CanFindTask};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Tower {
    room: RoomName,
}

impl CanFindTask for Tower {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
