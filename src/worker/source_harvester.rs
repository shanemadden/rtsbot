use log::*;
use serde::{Deserialize, Serialize};

use screeps::{constants::look, local::Position, objects::Store, prelude::*};

use crate::{game, task::Task, worker::Worker};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SourceHarvester {
    #[serde(rename = "s")]
    pub source_position: Position,
}

impl Worker for SourceHarvester {
    fn find_task(&self, _store: &Store) -> Task {
        match self.source_position.look_for(look::SOURCES) {
            Ok(sources) => match sources.get(0) {
                Some(source) => Task::HarvestEnergy(source.id()),
                None => Task::MoveToPosition(self.source_position, 1),
            },
            Err(_) => Task::MoveToPosition(self.source_position, 1),
        }
    }
}
