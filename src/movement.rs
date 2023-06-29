use screeps::{Direction, Position};

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

// struct for specifying where a creep wants to move and the options the pathfinder
// will need to know to get them there
#[derive(Debug, Clone, Copy)]
pub struct MovementGoal {
    pub goal: Position,
    pub goal_range: u8,
    pub priority: f64,
    pub profile: MovementProfile,
    pub avoid_creeps: bool,
}

// struct for tracking the current state of a moving creep
#[derive(Debug, Clone)]
pub struct PathState {
    pub stuck_count: u8,
    pub current_path: Vec<Direction>,
    pub current_path_progress: u32,
}

pub fn run_movement_and_remove_worker_refs(shard_state: &mut ShardState) {
    unimplemented!()
}
