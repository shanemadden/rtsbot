use std::collections::HashMap;

use screeps::{Direction, Position};

use crate::ShardState;

// enum for the different speeds available to creeps
#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
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
#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct MovementGoal {
    pub goal: Position,
    pub goal_range: u8,
    pub profile: MovementProfile,
    pub avoid_creeps: bool,
}

// struct for tracking the current state of a moving creep
#[derive(Debug, Clone)]
pub struct PathState {
    // track the goal this state moves towards - we'll confirm the creep
    // hasn't registered a new goal before using this cached state
    pub goal: MovementGoal,
    pub stuck_count: u8,
    pub next_position: Position,
    pub path: Vec<Direction>,
    pub path_progress: u32,
}

pub fn run_movement_and_remove_worker_refs(shard_state: &mut ShardState) {
    // creeps that are idle register themselves in this hashmap so that creeps
    // moving to their position can get them to swap positions as a simple
    // 'traffic management' mechanic
    let mut idle_creeps = HashMap::new();
    // and creeps that are moving register where they're looking to move here
    // when they do, so that we can look for idle creeps at that location
    // to swap with
    let mut move_creeps = HashMap::new();
    // loop through all workers, removing their reference for use
    // during this movement step (or simply discarded in the case
    // of worker roles that can't move)
    for worker_state in shard_state.worker_state.values_mut() {
        // take the reference out of the worker
        worker_state.worker_reference.take()
    }
}
