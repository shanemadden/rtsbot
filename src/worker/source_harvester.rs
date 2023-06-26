use serde::{Deserialize, Serialize};
use screeps::local::Position;
use crate::worker::{Task, CanFindTask};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct SourceHarvester {
    #[serde(rename = "s")]
    source_position: Position,
}

impl CanFindTask for SourceHarvester {
    fn find_task(&self) -> Task {
        unimplemented!()
    }
}
