use log::*;
use serde::{Deserialize, Serialize};

use screeps::{local::RoomName, objects::Store};

use crate::{game, task::Task, worker::Worker};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Upgrader {
    #[serde(rename = "r")]
    pub home_room: RoomName,
    #[serde(rename = "i")]
    pub id: u8,
}

impl Worker for Upgrader {
    fn find_task(&self, store: &Store) -> Task {
        unimplemented!()
    }
}
