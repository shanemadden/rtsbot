use serde::{Deserialize, Serialize};
use screeps::local::RoomName;
use crate::worker::{Task, CanFindTask};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Hauler {
    #[serde(rename = "r")]
    home_room: RoomName,
    #[serde(rename = "i")]
    id: u8,
}

impl CanFindTask for Hauler {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
