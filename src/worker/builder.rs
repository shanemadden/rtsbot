use crate::worker::{CanFindTask, Task};
use screeps::local::RoomName;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Builder {
    #[serde(rename = "r")]
    pub home_room: RoomName,
    #[serde(rename = "i")]
    pub id: u8,
}

impl CanFindTask for Builder {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
