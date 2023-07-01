use log::*;

use screeps::{constants::Direction, local::Position, pathfinder::SearchOptions};

use crate::{
    constants::*,
    movement::{callbacks::*, MovementProfile, PathState},
};

// struct for specifying where a creep wants to move and the options the pathfinder
// will need to know to get them there
#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct MovementGoal {
    pub goal_pos: Position,
    pub goal_range: u32,
    pub profile: MovementProfile,
    pub avoid_creeps: bool,
}

impl MovementGoal {
    pub fn find_path_to(&self, from_position: Position) -> PathState {
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
                }
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
                }
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
                }
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
                }
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
                }
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
                }
            }
        };

        // warn if we got an incomplete path, but still use it
        if search_result.incomplete() {
            warn!(
                "incomplete search! {} {} {}",
                search_result.ops(),
                search_result.cost(),
                self.goal_pos
            );
        }
        // start cursor from the current postion
        let mut cursor_pos = from_position;
        // load the path from the search result, which is Vec<Position>
        let positions = search_result.path();
        // make a Vec<Direction> for our stored path, which is more compact
        let mut steps = Vec::with_capacity(positions.len());
        for pos in positions {
            // skip storing this step if it's just a room boundary change
            // that'll happen automatically thanks to the edge tile's swap-every-tick
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
