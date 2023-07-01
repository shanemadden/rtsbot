use log::*;
use serde::{Deserialize, Serialize};

use screeps::{local::Position, objects::Store};

use crate::{game, task::Task, worker::Worker};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SourceHarvester {
    #[serde(rename = "s")]
    pub source_position: Position,
}

impl Worker for SourceHarvester {
    fn find_task(&self, store: &Store) -> Task {
        unimplemented!()
    }
}
