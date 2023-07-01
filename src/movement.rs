use log::*;
use std::collections::HashMap;

use screeps::{game, Direction, Position};

use crate::{
    worker::{Worker, WorkerReference},
    ShardState,
};

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
    pub last_position: Position,
    pub next_direction: Direction,
    pub path: Vec<Direction>,
    pub path_progress: usize,
}

impl PathState {
    fn check_if_moved_and_update_pos(&mut self, current_position: Position) {
        // first we'll check if the creep actually moved as we intended last tick,
        // incrementing the path_progress if so (and incrementing the stuck_count if not)
        if current_position == (self.last_position + self.next_direction) {
            // we've moved as intended (yay); let's update the current position..
            self.last_position = current_position;
            // ..and bump the cursor for the next move
            self.path_progress += 1;
            // ..and reset the stuck count
            self.stuck_count = 0;
        } else if current_position == self.last_position {
            // didn't move, simply increment the stuck counter
            self.stuck_count += 1;
        } else {
            // we're not in the right spot. If we're in a different position than we were
            // last tick, something weird is going on (possibly stuck on an exit tile or portal) -
            // we want to repath in this case, so send the stuck count way up to trigger that
            self.stuck_count = u8::MAX;
        }
    }
}

impl WorkerReference {
    fn move_with_path(
        &self,
        mut path_state: PathState,
        current_position: Position,
        moving_creeps: &mut HashMap<Position, Direction>,
    ) -> Option<PathState> {
        match path_state.path.get(path_state.path_progress) {
            Some(direction) => match self {
                WorkerReference::Creep(creep) => {
                    // do the actual move in the intended direction
                    let _ = creep.move_direction(*direction);
                    // set next_direction so we can detect if this worked next tick
                    path_state.next_direction = *direction;
                    // insert a key of the position the creep intends to move to,
                    // and a value of the direction this creep is moving (so a creep)
                    // at the target position can infer which direction they should move to swap
                    moving_creeps.insert(current_position + *direction, *direction);
                    Some(path_state)
                }
                _ => {
                    warn!("can't move worker in move_with_path?");
                    None
                }
            },
            None => None,
        }
    }
}

impl MovementGoal {
    fn find_path_to(&self) -> PathState {
        unimplemented!();
    }
}

pub fn run_movement_and_remove_worker_refs(shard_state: &mut ShardState) {
    // creeps that are idle register themselves in this hashmap so that creeps
    // moving to their position can get them to swap positions as a simple
    // 'traffic management' mechanic (but pretty durable, absent pull() trains or immobile creeps)
    let mut idle_creeps = HashMap::new();

    // and creeps that are moving register where they're looking to move here
    // when they do, so that we can look for idle creeps at that location
    // to swap with
    let mut moving_creeps = HashMap::new();

    // check if CPU is high this tick or the bucket is low, we'll skip finding new paths if so
    let tick_cpu = game::cpu::get_used();
    let bucket_cpu = game::cpu::bucket();
    let cpu_critical = if tick_cpu > crate::constants::HIGH_CPU_THRESHOLD {
        warn!(
            "CPU usage high, will skip finding fresh paths: {}",
            tick_cpu
        );
        true
    } else if bucket_cpu < crate::constants::LOW_BUCKET_THRESHOLD {
        warn!(
            "CPU bucket low, will skip finding fresh paths: {}",
            bucket_cpu
        );
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
            if worker_state.role.can_move() && worker_reference.fatigue() == 0 {
                // it's a role that can move, let's consider it for movement
                let position = worker_reference.pos();
                // it can move - check if it has somewhere to be, and mark it as idle if not
                if let Some(movement_goal) = worker_state.movement_goal.take() {
                    // we have a goal; first check if it's met
                    if position.get_range_to(movement_goal.goal_pos)
                        <= movement_goal.goal_range as u32
                    {
                        // goal is met! unset the path_state if there is one and idle
                        worker_state.path_state = None;
                        idle_creeps.insert(position, worker_reference);
                    } else {
                        // goal isn't met - let's see if there's a cached path
                        let path_needed =
                            if let Some(mut path_state) = worker_state.path_state.take() {
                                // first call the function that updates the current position (or the stuck count)
                                path_state.check_if_moved_and_update_pos(position);

                                if path_state.goal == movement_goal
                                    && path_state.stuck_count
                                        <= crate::constants::STUCK_REPATH_THRESHOLD
                                {
                                    // still has the same goal as the cached path; we're ok
                                    // to simply move, retaining the path unless it's not returned
                                    worker_state.path_state = worker_reference.move_with_path(
                                        path_state,
                                        position,
                                        &mut moving_creeps,
                                    );
                                    false
                                } else {
                                    // the goal has changed or we're stuck - mark pathing as needed and
                                    // ditch this state
                                    true
                                }
                            } else {
                                // no cached path found, mark as needed
                                true
                            };

                        // if we need to path and we're in a CPU state to do it, do so
                        if path_needed && !cpu_critical {
                            let mut path_state = movement_goal.find_path_to();
                            worker_state.path_state = worker_reference.move_with_path(
                                path_state,
                                position,
                                &mut moving_creeps,
                            );
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
