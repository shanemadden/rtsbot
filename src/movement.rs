use log::*;
use std::collections::HashMap;

use screeps::{
    constants::{Direction, StructureType},
    enums::StructureObject,
    game, find,
    local::{LocalCostMatrix, Position, RoomName},
    pathfinder::{MultiRoomCostResult, SearchOptions},
    prelude::*,
};

use crate::{
    constants::*,
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
    pub goal_range: u32,
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
            // we've moved as intended (yay); let's update the last good position..
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

fn callback_standard(room_name: RoomName) -> MultiRoomCostResult {
    let mut new_matrix = LocalCostMatrix::new();
    match screeps::game::rooms().get(room_name) {
        Some(room) => {
            for structure in room.find(find::STRUCTURES, None) {
                let pos = structure.pos();
                match structure {
                    // ignore roads for creeps not needing 'em
                    StructureObject::StructureRoad(_) => {},
                    // containers walkable
                    StructureObject::StructureContainer(_) => {},
                    StructureObject::StructureWall(_) => {
                        new_matrix.set(pos.xy(), 0xff);
                    },
                    StructureObject::StructureRampart(rampart) => {
                        // we could check for and path across public ramparts
                        // (and need to do so if we want to enhance this bot to be able
                        // to cross an ally's public ramparts - but for now, simply don't trust 'em
                        if !rampart.my() {
                            new_matrix.set(pos.xy(), 0xff);
                        }
                    },
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    },
                }
            }

            for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                let pos = csite.pos();
                match csite.structure_type() {
                    // walkable structure types
                    StructureType::Container | StructureType::Road | StructureType::Rampart => {},
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    },
                }
            }
        }
        // can't see the room; terrain matrix is fine
        None => {}
    }
    MultiRoomCostResult::CostMatrix(new_matrix.into())
}

fn callback_roads(room_name: RoomName) -> MultiRoomCostResult {
    let mut new_matrix = LocalCostMatrix::new();
    match screeps::game::rooms().get(room_name) {
        Some(room) => {
            for structure in room.find(find::STRUCTURES, None) {
                let pos = structure.pos();
                match structure {
                    StructureObject::StructureRoad(_) => {
                        if new_matrix.get(pos.xy()) == 0 {
                            new_matrix.set(pos.xy(), 0x01);
                        }
                    },
                    // containers walkable
                    StructureObject::StructureContainer(_) => {}
                    StructureObject::StructureWall(_) => {
                        new_matrix.set(pos.xy(), 0xff);
                    },
                    StructureObject::StructureRampart(rampart) => {
                        // we could check for and path across public ramparts
                        // (and need to do so if we want to enhance this bot to be able
                        // to cross an ally's public ramparts - but for now, simply don't trust 'em
                        if !rampart.my() {
                            new_matrix.set(pos.xy(), 0xff);
                        }
                    },
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    },
                }
            }

            for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                let pos = csite.pos();
                match csite.structure_type() {
                    // walkable structure types
                    StructureType::Container | StructureType::Road | StructureType::Rampart => {},
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    },
                }
            }
        }
        // can't see the room; terrain matrix is fine
        None => {}
    }
    MultiRoomCostResult::CostMatrix(new_matrix.into())
}


fn callback_standard_avoiding_creeps(room_name: RoomName) -> MultiRoomCostResult {
    let mut new_matrix = LocalCostMatrix::new();
    match screeps::game::rooms().get(room_name) {
        Some(room) => {
            for structure in room.find(find::STRUCTURES, None) {
                let pos = structure.pos();
                match structure {
                    // ignore roads for creeps not needing 'em
                    StructureObject::StructureRoad(_) => {},
                    // containers walkable
                    StructureObject::StructureContainer(_) => {},
                    StructureObject::StructureWall(_) => {
                        new_matrix.set(pos.xy(), 0xff);
                    },
                    StructureObject::StructureRampart(rampart) => {
                        // we could check for and path across public ramparts
                        // (and need to do so if we want to enhance this bot to be able
                        // to cross an ally's public ramparts - but for now, simply don't trust 'em
                        if !rampart.my() {
                            new_matrix.set(pos.xy(), 0xff);
                        }
                    },
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    },
                }
            }

            for creep in room.find(find::CREEPS, None) {
                let pos = creep.pos();
                new_matrix.set(pos.xy(), 0x20);
            }

            for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                let pos = csite.pos();
                match csite.structure_type() {
                    // walkable structure types
                    StructureType::Container | StructureType::Road | StructureType::Rampart => {},
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    },
                }
            }
        }
        // can't see the room; terrain matrix is fine
        None => {}
    }
    MultiRoomCostResult::CostMatrix(new_matrix.into())
}

fn callback_roads_avoiding_creeps(room_name: RoomName) -> MultiRoomCostResult {
    let mut new_matrix = LocalCostMatrix::new();
    match screeps::game::rooms().get(room_name) {
        Some(room) => {
            for structure in room.find(find::STRUCTURES, None) {
                let pos = structure.pos();
                match structure {
                    StructureObject::StructureRoad(_) => {
                        if new_matrix.get(pos.xy()) == 0 {
                            new_matrix.set(pos.xy(), 0x01);
                        }
                    },
                    // containers walkable
                    StructureObject::StructureContainer(_) => {}
                    StructureObject::StructureWall(_) => {
                        new_matrix.set(pos.xy(), 0xff);
                    },
                    StructureObject::StructureRampart(rampart) => {
                        // we could check for and path across public ramparts
                        // (and need to do so if we want to enhance this bot to be able
                        // to cross an ally's public ramparts - but for now, simply don't trust 'em
                        if !rampart.my() {
                            new_matrix.set(pos.xy(), 0xff);
                        }
                    },
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    },
                }
            }

            for creep in room.find(find::CREEPS, None) {
                let pos = creep.pos();
                new_matrix.set(pos.xy(), 0x20);
            }


            for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                let pos = csite.pos();
                match csite.structure_type() {
                    // walkable structure types
                    StructureType::Container | StructureType::Road | StructureType::Rampart => {},
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    },
                }
            }
        }
        // can't see the room; terrain matrix is fine
        None => {}
    }
    MultiRoomCostResult::CostMatrix(new_matrix.into())
}


impl MovementGoal {
    fn find_path_to(&self, from_position: Position) -> PathState {
        let search_result = if self.avoid_creeps {
            match self.profile {
                // creep that moves at full speed over swamp, treat swamps as the same as plains
                MovementProfile::SwampFiveToOne => {
                    let options = SearchOptions::new(callback_standard)
                        .max_ops(MAX_OPS)
                        .max_rooms(MAX_ROOMS)
                        .swamp_cost(1)
                        .heuristic_weight(1.0);
                    screeps::pathfinder::search(
                        from_position,
                        self.goal_pos,
                        self.goal_range,
                        Some(options),
                    )
                },
                MovementProfile::PlainsOneToOne => {
                    let options = SearchOptions::new(callback_standard)
                        .max_ops(MAX_OPS)
                        .max_rooms(MAX_ROOMS)
                        .heuristic_weight(1.0);
                    screeps::pathfinder::search(
                        from_position,
                        self.goal_pos,
                        self.goal_range,
                        Some(options),
                    )
                },
                // double the cost of swamps and plains to allow roads to be lowest
                MovementProfile::RoadsOneToTwo => {
                    let options = SearchOptions::new(callback_roads)
                        .max_ops(MAX_OPS)
                        .max_rooms(MAX_ROOMS)
                        .plain_cost(2)
                        .swamp_cost(10)
                        .heuristic_weight(1.0);
                    screeps::pathfinder::search(
                        from_position,
                        self.goal_pos,
                        self.goal_range,
                        Some(options),
                    )
                },
            }
        } else {
            match self.profile {
                // creep that moves at full speed over swamp, treat swamps as the same as plains
                MovementProfile::SwampFiveToOne => {
                    let options = SearchOptions::new(callback_standard_avoiding_creeps)
                        .max_ops(MAX_OPS)
                        .max_rooms(MAX_ROOMS)
                        .swamp_cost(1)
                        .heuristic_weight(1.0);
                    screeps::pathfinder::search(
                        from_position,
                        self.goal_pos,
                        self.goal_range,
                        Some(options),
                    )
                },
                MovementProfile::PlainsOneToOne => {
                    let options = SearchOptions::new(callback_standard_avoiding_creeps)
                        .max_ops(MAX_OPS)
                        .max_rooms(MAX_ROOMS)
                        .heuristic_weight(1.0);
                    screeps::pathfinder::search(
                        from_position,
                        self.goal_pos,
                        self.goal_range,
                        Some(options),
                    )
                },
                // double the cost of swamps and plains to allow roads to be lowest
                MovementProfile::RoadsOneToTwo => {
                    let options = SearchOptions::new(callback_roads_avoiding_creeps)
                        .max_ops(MAX_OPS)
                        .max_rooms(MAX_ROOMS)
                        .plain_cost(2)
                        .swamp_cost(10)
                        .heuristic_weight(1.0);
                    screeps::pathfinder::search(
                        from_position,
                        self.goal_pos,
                        self.goal_range,
                        Some(options),
                    )
                },
            }
        };

        if search_result.incomplete() {
            warn!(
                "incomplete search! {} {} {}",
                search_result.ops(),
                search_result.cost(),
                self.goal_pos
            );
        }
        let positions = search_result.path();
        let mut steps = vec![];

        let mut cursor_pos = from_position;
        for pos in positions {
            // skip storing this step if it's just a room boundary change
            // that'll happen automatically thanks to the edge tile
            if pos.room_name() == cursor_pos.room_name() {
                match pos.get_direction_to(cursor_pos) {
                    Some(v) => {
                        // store the inverse of the direction to cursor_pos,
                        // since it's earlier in the path
                        let v = -v;
                        steps.push(v);
                    }
                    None => {
                        warn!("direction failure?");
                        break;
                    }
                }
            }
            cursor_pos = pos;
        }

        PathState {
            goal: *self,
            stuck_count: 0,
            last_position: from_position,
            // in the rare case we got a zero-step incomplete path, just
            // mark top as the direction we're moving; the path will just fail next tick
            next_direction: *steps.get(0).unwrap_or(&Direction::Top),
            path: steps,
            path_progress: 0,
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
                    if position.get_range_to(movement_goal.goal_pos)
                        <= movement_goal.goal_range as u32
                    {
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
        match idle_creeps.get(&dest_pos) {
            // use the `std::ops::Neg` implementation to get the opposite direction
            Some(worker_reference) => worker_reference.swap_move(-*moving_direction),
            None => {}
        }
    }
}
