use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use screeps::{
    constants::look,
    constants::Part,
    local::Position,
    objects::{Store, StructureSpawn},
    prelude::*,
};

use crate::{role::WorkerRole, task::Task, worker::Worker};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SourceHarvester {
    #[serde(rename = "s", with = "screeps::local::serde_position_packed")]
    pub source_position: Position,
}

impl Worker for SourceHarvester {
    fn find_task(&self, _store: &Store, _worker_roles: &HashSet<WorkerRole>) -> Task {
        match self.source_position.look_for(look::SOURCES) {
            Ok(sources) => match sources.first() {
                Some(source) => Task::HarvestEnergyForever(source.id()),
                None => Task::MoveToPosition(self.source_position, 1),
            },
            Err(_) => Task::MoveToPosition(self.source_position, 1),
        }
    }

    fn get_body_for_creep(&self, _spawn: &StructureSpawn) -> Vec<Part> {
        use Part::*;
        vec![Move, Move, Move, Work, Work, Work, Work, Work]
    }
}
