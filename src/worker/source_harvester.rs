use crate::{worker::Worker, task::Task};
use screeps::local::Position;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SourceHarvester {
    #[serde(rename = "s")]
    pub source_position: Position,
}

impl Worker for SourceHarvester {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
