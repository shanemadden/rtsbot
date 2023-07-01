use std::collections::HashMap;
use log::*;

use screeps::{game, Direction, Position};

use crate::{ShardState, worker::{Worker, WorkerReference}};

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
    pub goal_pos: Position,
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
    pub current_position: Position,
    pub next_direction: Direction,
    pub path: Vec<Direction>,
    pub path_progress: u32,
}

fn move_by_path(worker_reference: &WorkerReference, path_state: &mut PathState, moving_creeps: &mut HashMap<(Direction, Position), WorkerReference>) {
    
}

fn find_path_to_goal(movement_goal: &MovementGoal) -> PathState {
    unimplemented!();
}

pub fn run_movement_and_remove_worker_refs(shard_state: &mut ShardState) {
    // creeps that are idle register themselves in this hashmap so that creeps
    // moving to their position can get them to swap positions as a simple
    // 'traffic management' mechanic
    let mut idle_creeps = HashMap::new();

    // and creeps that are moving register where they're looking to move here
    // when they do, so that we can look for idle creeps at that location
    // to swap with
    let mut moving_creeps = HashMap::new();

    // check if CPU is high this tick or the bucket is low, we'll skip finding new paths if so
    let tick_cpu = game::cpu::get_used();
    let bucket_cpu = game::cpu::bucket();
    let cpu_critical = if tick_cpu > crate::constants::HIGH_CPU_THRESHOLD {
        warn!("CPU usage high, will skip finding fresh paths: {}", tick_cpu);
        true
    } else if bucket_cpu < crate::constants::LOW_BUCKET_THRESHOLD {
        warn!("CPU bucket low, will skip finding fresh paths: {}", bucket_cpu);
        true
    } else {
        false
    };

    // loop through all workers, removing their reference for use
    // during this movement step (or simply discarded in the case
    // of worker roles that can't move)
    for worker_state in shard_state.worker_state.values_mut() {
        // take the reference out of the worker
        if let Some(worker_reference) = worker_state.worker_reference.take() {
            // if the worker can't move, that's all we needed to do as end-of-tick cleanup
            if worker_state.role.can_move() {
                // it's a role that can move, let's consider it for movement
                let position = worker_reference.pos();
                // it can move - check if it has somewhere to be, and mark it as idle if not
                if let Some(movement_goal) = worker_state.movement_goal.take() {
                    // we have a goal; first check if it's met
                    if position.get_range_to(movement_goal.goal_pos) <= movement_goal.goal_range as u32 {
                        // goal is met! unset the path_state if there is one and idle
                        worker_state.path_state = None;
                        idle_creeps.insert(position, worker_reference);
                    } else {
                        // goal isn't met - let's see if there's a cached path
                        let mut path_needed = false;
                        if let Some(mut path_state) = worker_state.path_state.take() {
                            if path_state.goal == movement_goal && path_state.stuck_count < crate::constants::STUCK_REPATH_THRESHOLD {
                                // still has the same goal as the cached path; we're ok
                                // to simply move, then retain the path state for the next tick
                                move_by_path(&worker_reference, &mut path_state, &mut moving_creeps);
                                worker_state.path_state = Some(path_state);
                            } else {
                                // the goal has changed or we're stuck - mark pathing as needed and
                                // don't keep state
                                path_needed = true;
                            }
                        } else {
                            path_needed = true;
                        }

                        // if we need to path and we're in a CPU state to do it, do so
                        if path_needed && !cpu_critical {
                            let mut path_state = find_path_to_goal(&movement_goal);
                            move_by_path(&worker_reference, &mut path_state, &mut moving_creeps);
                            worker_state.path_state = Some(path_state);
                        }

                        // put the goal back that we took, since the goal isn't yet met
                        worker_state.movement_goal = Some(movement_goal);
                    }
                } else {
                    // no goal, mark as idle!
                    idle_creeps.insert(position, worker_reference);
                }
            }
        } else {
            warn!("worker with no reference in move step!");
            continue;
        }
    }

    // look for idle creeps where we actively have creeps saying they intend to move

}
