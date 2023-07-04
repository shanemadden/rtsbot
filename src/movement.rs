use log::*;
use std::collections::HashMap;

use screeps::{
    constants::Direction,
    game,
    local::Position,
    visual::{LineDrawStyle, PolyStyle, RoomVisual},
};

use crate::{
    constants::*,
    worker::{Worker, WorkerReference},
    ShardState,
};

mod callbacks;
mod movement_goal;
mod path_state;

pub use callbacks::*;
pub use movement_goal::MovementGoal;
pub use path_state::PathState;

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

impl WorkerReference {
    fn move_with_path(
        &self,
        mut path_state: PathState,
        current_position: Position,
        moving_creeps: &mut HashMap<Position, Direction>,
    ) -> Option<PathState> {
        #[cfg(feature = "path-visuals")]
        {
            let mut points = vec![];
            let mut cursor_pos = current_position;
            for step in path_state.path[path_state.path_progress..].iter() {
                cursor_pos = cursor_pos + *step;
                if cursor_pos.room_name() != current_position.room_name() {
                    break;
                }
                points.push((cursor_pos.x().u8() as f32, cursor_pos.y().u8() as f32));
            }
            RoomVisual::new(Some(current_position.room_name())).poly(
                points,
                Some(
                    PolyStyle::default()
                        .fill("transparent")
                        .stroke("#f00")
                        .line_style(LineDrawStyle::Dashed)
                        .stroke_width(0.15)
                        .opacity(0.5),
                ),
            );
        }

        match path_state.path.get(path_state.path_progress) {
            Some(direction) => match self {
                WorkerReference::Creep(creep) => {
                    // do the actual move in the intended direction
                    let _ = creep.move_direction(*direction);
                    // set next_direction so we can detect if this worked next tick
                    path_state.next_direction = *direction;
                    // insert a key of the position the creep intends to move to,
                    // and a value of the direction this creep is moving (so a creep
                    // at the target position can infer which direction they should move to swap)
                    moving_creeps.insert(current_position + *direction, *direction);
                    Some(path_state)
                }
                _ => {
                    warn!("can't move worker in move_with_path");
                    None
                }
            },
            None => None,
        }
    }

    fn swap_move(&self, direction: Direction) {
        match self {
            WorkerReference::Creep(creep) => {
                let _ = creep.move_direction(direction);
                let _ = creep.say(format!("{}", direction).as_str(), true);
            }
            _ => warn!("can't move worker in swap_move"),
        }
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
    let cpu_critical = if tick_cpu > HIGH_CPU_THRESHOLD {
        warn!(
            "CPU usage high, will skip finding fresh paths: {}",
            tick_cpu
        );
        true
    } else if bucket_cpu < LOW_BUCKET_THRESHOLD {
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
                    if position.get_range_to(movement_goal.goal_pos) <= movement_goal.goal_range {
                        // goal is met! unset the path_state if there is one and idle
                        worker_state.path_state = None;
                        idle_creeps.insert(position, worker_reference);
                    } else {
                        // goal isn't met - let's see if there's a cached path that seems valid
                        let path_needed =
                            if let Some(mut path_state) = worker_state.path_state.take() {
                                // first call the function that updates the current position
                                // (or the stuck count if we didn't move)
                                path_state.check_if_moved_and_update_pos(position);

                                if path_state.goal == movement_goal
                                    && path_state.stuck_count <= STUCK_REPATH_THRESHOLD
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
                                    // the goal has changed or we're stuck - mark pathing as needed!
                                    true
                                }
                            } else {
                                // no cached path found, mark as needed
                                true
                            };

                        // if we need to path and we're in a CPU state to do it, do so
                        if path_needed && !cpu_critical {
                            let path_state = movement_goal.find_path_to(position);
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
    for (dest_pos, moving_direction) in moving_creeps.iter() {
        if let Some(worker_reference) = idle_creeps.get(dest_pos) {
            worker_reference.swap_move(-*moving_direction)
        }
    }
}
