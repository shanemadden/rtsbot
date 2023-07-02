use std::collections::HashSet;
use log::*;
use serde::{Deserialize, Serialize};

use screeps::{local::RoomName, objects::Store};

use crate::{constants::*, game, task::Task, worker::{Worker, WorkerRole}};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub room: RoomName,
}

impl Worker for Spawn {
    fn find_task(&self, _store: &Store, worker_roles: &HashSet<WorkerRole>) -> Task {
        // for each role variant we want a creep occupying, check
        // if a worker exists; if not, that's the creep we'll pick to spawn next

        let room = game::rooms().get(self.room).expect("expected room for active spawn");
        let repair_watermark = match room.controller().expect("expected controller in room with spawn").level() {
            1 => REPAIR_WATERMARK_RCL_1,
            2 => REPAIR_WATERMARK_RCL_2,
            3 => REPAIR_WATERMARK_RCL_3,
            4 => REPAIR_WATERMARK_RCL_4,
            5 => REPAIR_WATERMARK_RCL_5,
            6 => REPAIR_WATERMARK_RCL_6,
            7 => REPAIR_WATERMARK_RCL_7,
            _ => REPAIR_WATERMARK_RCL_8,
        };

        // determine if we should spawn a builder
        let mut should_spawn_builder = false;

        // check for construction sites
        // check for repairable structures
        unimplemented!();

        //if !worker_roles.contains(WorkerRole::Builder(Builder{}))
    }

    fn can_move(&self) -> bool {
        false
    }
}
