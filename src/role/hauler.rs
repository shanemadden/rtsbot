use log::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use screeps::{
    constants::{find, Part, ResourceType},
    enums::StructureObject,
    local::RoomName,
    objects::{Room, Store, Structure, StructureSpawn},
    prelude::*,
};

use crate::{constants::*, game, role::WorkerRole, task::Task, worker::Worker};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Hauler {
    #[serde(rename = "r")]
    pub home_room: RoomName,
    #[serde(rename = "i")]
    pub id: u8,
}

impl Worker for Hauler {
    fn find_task(&self, store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        match game::rooms().get(self.home_room) {
            Some(room) => {
                if store.get_used_capacity(Some(ResourceType::Energy)) > 0 {
                    find_delivery_target(&room)
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

    fn get_body_for_creep(&self, spawn: &StructureSpawn) -> Vec<Part> {
        // scale the creep to larger depending on how much capacity we have available
        let max_energy_avail = spawn.room().expect("spawn to have room").energy_capacity_available();
        let multiplier = std::cmp::min(max_energy_avail / HAULER_COST_PER_MULTIPLIER, HAULER_MAX_MULTIPLIER);

        [Part::Carry, Part::Carry, Part::Move].repeat(multiplier as usize)
    }
}

fn find_energy(room: &Room) -> Task {
    // check for energy on the ground of sufficient quantity to care about
    for resource in room.find(find::DROPPED_RESOURCES, None) {
        if resource.resource_type() == ResourceType::Energy
            && resource.amount() >= HAULER_ENERGY_PICKUP_THRESHOLD
        {
            return Task::TakeFromResource(resource.id());
        }
    }

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

        if store.get_used_capacity(Some(ResourceType::Energy)) >= HAULER_ENERGY_WITHDRAW_THRESHOLD {
            return Task::TakeFromStructure(structure.as_structure().id(), ResourceType::Energy);
        }
    }

    Task::IdleUntil(game::time() + NO_TASK_IDLE_TICKS)
}

fn find_delivery_target(room: &Room) -> Task {
    // check structures - we'll do a pass looking for high priority structures
    // like spawns and extensions and towers before we check terminal and storage -
    // but we'll store their references here as we come accoss them
    let mut maybe_storage = None;
    let mut maybe_terminal = None;

    for structure in room.find(find::STRUCTURES, None) {
        let (store, structure) = match structure {
            // for the three object types that are important to fill, snag their store then cast
            // them right back to StructureObject
            StructureObject::StructureSpawn(ref o) => (o.store(), structure),
            StructureObject::StructureExtension(ref o) => (o.store(), structure),
            StructureObject::StructureTower(ref o) => (o.store(), structure),
            // don't want to look at these types in this iteration, in case
            // one of the covered priority types is later in the vec
            StructureObject::StructureStorage(o) => {
                maybe_storage = Some(o);
                continue;
            }
            StructureObject::StructureTerminal(o) => {
                maybe_terminal = Some(o);
                continue;
            }
            _ => {
                // we don't want to look at this!
                continue;
            }
        };

        if store.get_free_capacity(Some(ResourceType::Energy)) > 0 {
            return Task::DeliverToStructure(structure.as_structure().id(), ResourceType::Energy);
        }
    }

    // check the terminal if we found one
    if let Some(terminal) = maybe_terminal {
        if terminal
            .store()
            .get_used_capacity(Some(ResourceType::Energy))
            < TERMINAL_ENERGY_TARGET
        {
            return Task::DeliverToStructure(
                terminal.id().into_type::<Structure>(),
                ResourceType::Energy,
            );
        }
    }

    // and finally check the storage
    if let Some(storage) = maybe_storage {
        return Task::DeliverToStructure(
            storage.id().into_type::<Structure>(),
            ResourceType::Energy,
        );
    }

    Task::IdleUntil(game::time() + NO_TASK_IDLE_TICKS)
}
