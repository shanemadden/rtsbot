use std::collections::HashSet;
use log::*;
use serde::{Deserialize, Serialize};

use screeps::{constants::Part, constants::find, game, local::RoomName, objects::{Store, StructureSpawn}};

use crate::{constants::*, task::{Task}, role::*};

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
        let mut should_ensure_builder = false;

        // check for construction sites
        if room.find(find::MY_CONSTRUCTION_SITES, None).len() > 0 {
            should_ensure_builder = true;
        } else {
            // check for repairable structures

            for structure_object in room.find(find::STRUCTURES, None) {
                let structure = structure_object.as_structure();
                let hits = structure.hits();
                let hits_max = structure.hits_max();

                // if hits_max is 0, it's indestructable
                if hits_max != 0 {
                    // if the hits are below our 'watermark' to repair to
                    // as well as less than half of this struture's max, repair!
                    if hits < repair_watermark && hits * 2 < hits_max {
                        should_ensure_builder = true;
                        break
                    }
                }
            }
        }

        if should_ensure_builder {
            let role = WorkerRole::Builder(Builder {
                home_room: self.room,
                repair_watermark
            });
            if !worker_roles.contains(&role) {
                return Task::SpawnCreep(role)
            }
        }

        // todo: the remaining roles
        Task::IdleUntil(u32::MAX)
    }

    fn get_body_for_creep(&self, _spawn: &StructureSpawn) -> Vec<Part> {
        panic!("can't spawn creep for spawn")
    }

    fn can_move(&self) -> bool {
        false
    }
}
