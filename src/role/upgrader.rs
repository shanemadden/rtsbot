use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use screeps::{
    constants::{find, Part, ResourceType},
    enums::StructureObject,
    game,
    local::RoomName,
    objects::{Room, Store, StructureSpawn},
    prelude::*,
};

use crate::{constants::*, role::WorkerRole, task::Task, worker::Worker};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Upgrader {
    #[serde(rename = "r")]
    pub home_room: RoomName,
    #[serde(rename = "i")]
    pub id: u8,
}

impl Worker for Upgrader {
    fn find_task(&self, store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        match game::rooms().get(self.home_room) {
            Some(room) => {
                if store.get_used_capacity(Some(ResourceType::Energy)) > 0 {
                    find_upgrade_task(&room)
                } else {
                    find_energy(&room)
                }
            }
            None => {
                warn!("couldn't see room for task find, must be an orphan");
                Task::IdleUntil(u32::MAX)
            }
        }
    }

    fn get_body_for_creep(&self, _spawn: &StructureSpawn) -> Vec<Part> {
        use Part::*;
        vec![Move, Move, Carry, Work]
    }
}

fn find_upgrade_task(room: &Room) -> Task {
    if let Some(controller) = room.controller() {
        Task::Upgrade(controller.id())
    } else {
        Task::IdleUntil(game::time() + NO_TASK_IDLE_TICKS)
    }
}

fn find_energy(room: &Room) -> Task {
    // check structures - containers and terminals only, don't want
    // to have these taking from spawns or extensions!
    for structure in room.find(find::STRUCTURES, None) {
        let store = match &structure {
            StructureObject::StructureContainer(o) => o.store(),
            StructureObject::StructureStorage(o) => o.store(),
            StructureObject::StructureTerminal(o) => o.store(),
            _ => {
                // we don't want to look at this!
                continue;
            }
        };

        if store.get_used_capacity(Some(ResourceType::Energy)) >= UPGRADER_ENERGY_WITHDRAW_THRESHOLD
        {
            return Task::TakeFromStructure(structure.as_structure().id(), ResourceType::Energy);
        }
    }

    // look for sources with energy we can harvest as a last resort
    if let Some(source) = room.find(find::SOURCES_ACTIVE, None).into_iter().next() {
        return Task::HarvestEnergyUntilFull(source.id());
    }

    Task::IdleUntil(game::time() + NO_TASK_IDLE_TICKS)
}
