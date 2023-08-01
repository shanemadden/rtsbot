use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use screeps::{
    constants::find,
    constants::Part,
    game,
    local::RoomName,
    objects::{Store, StructureSpawn},
};

use crate::{constants::*, role::*, task::Task};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub room: RoomName,
}

impl Worker for Spawn {
    fn find_task(&self, _store: &Store, worker_roles: &HashSet<WorkerRole>) -> Task {
        // for each role variant we want a creep occupying, check
        // if a worker exists; if not, that's the creep we'll pick to spawn next

        let room = game::rooms()
            .get(self.room)
            .expect("expected room for active spawn");
        let room_level = room
            .controller()
            .expect("expected controller in room with spawn")
            .level();

        if room_level < 3 {
            // just make sure there's a bunch of startup creeps or else return idle
            for i in 0..STARTUP_RCL1_COUNT_TARGET {
                // check up until our max count, ensuring each one exists
                let startup_role = WorkerRole::Startup(Startup {
                    home_room: self.room,
                    id: i,
                });
                if !worker_roles.contains(&startup_role) {
                    return Task::SpawnCreep(startup_role);
                }
            }

            // we only want starter creeps, idle
            return Task::IdleUntil(game::time() + NO_TASK_IDLE_TICKS)
        }
        
        let repair_watermark = match room_level {
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
        if !room.find(find::MY_CONSTRUCTION_SITES, None).is_empty() {
            should_ensure_builder = true;
        } else {
            // check for repairable structuresa
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
                        break;
                    }
                }
            }
        }

        if should_ensure_builder {
            let builder_role = WorkerRole::Builder(Builder {
                home_room: self.room,
                repair_watermark,
            });
            if !worker_roles.contains(&builder_role) {
                return Task::SpawnCreep(builder_role);
            }
        }

        // todo - right now we're getting stuck on the hauler task since it can't scale down and the builders don't 
        // supply the spawn structures. need to split into emergency builders which only happen at low RCL or in emergencies
        // which supply the spawns then later RCL builders that only build and repair

        // ... then just wrap up getting the source harvesters going at certain RCLs and I think this example bot is
        // pretty much 'done'
        for i in 0..HAULER_COUNT_TARGET {
            // check up until our max count, ensuring each one exists
            let hauler_role = WorkerRole::Hauler(Hauler {
                home_room: self.room,
                id: i,
            });
            if !worker_roles.contains(&hauler_role) {
                return Task::SpawnCreep(hauler_role);
            }
        }

        for i in 0..UPGRADER_COUNT_TARGET {
            // check up until our max count, ensuring each one exists
            let upgrader_role = WorkerRole::Upgrader(Upgrader {
                home_room: self.room,
                id: i,
            });
            if !worker_roles.contains(&upgrader_role) {
                return Task::SpawnCreep(upgrader_role);
            }
        }

        // todo source harvesters too, look at sources on room if RCL high enough

        // last resort, idle
        Task::IdleUntil(game::time() + NO_TASK_IDLE_TICKS)
    }

    fn get_body_for_creep(&self, _spawn: &StructureSpawn) -> Vec<Part> {
        panic!("can't spawn creep for spawn")
    }

    fn can_move(&self) -> bool {
        false
    }
}
