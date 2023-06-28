use screeps::{Position, Direction};

use crate::ShardState;

// enum for the different speeds available to creeps
#[derive(Debug, Clone, Copy)]
pub enum MovementProfile {
    // can move at full speed on swamp (either 5:1 move parts ratio, or
    // all parts are move/empty carry)
    SwampFiveToOne,
    // can move at full speed on plains (1:1 move ratio)
    PlainsOneToOne,
    // can only move once per tick on roads, weight them appropriately
    RoadsOneToTwo,
    // immovable; this creep is doing something important or has no move parts,
    // other creeps should path around
    Obstacle,
}

// enum to track the state of each creep actively moving (or actively standing still)
// 
#[derive(Debug, Clone)]
pub struct MovementState {
    pub goal: Position,
    pub priority: f64,
    pub profile: MovementProfile,
    pub goal_range: u8,
    pub avoid_creeps: bool,
    pub stuck_count: u8,
    pub current_path: Option<(Vec<Direction>, u32)>
}

pub fn run_movement_and_remove_worker_refs(shard_state: &mut ShardState) {
    unimplemented!()
}
