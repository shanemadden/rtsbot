use screeps::{constants::Direction, local::Position};

use crate::movement::MovementGoal;

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
    pub fn check_if_moved_and_update_pos(&mut self, current_position: Position) {
        // first we'll check if the creep actually moved as we intended last tick,
        // incrementing the path_progress if so (and incrementing the stuck_count if not)
        if current_position == (self.last_position + self.next_direction) {
            // we've moved as intended (yay); let's update the last good position..
            self.last_position = current_position;
            // ..and bump the cursor for the next move..
            self.path_progress += 1;
            // ..and reset the stuck count
            self.stuck_count = 0;
        } else if current_position == self.last_position {
            // didn't move, simply increment the stuck counter
            self.stuck_count += 1;
        } else {
            // we're not in the right spot. If we're in a different position than we were
            // last tick, something weird is going on (possibly stuck on an exit tile or portal) -
            // we want to repath in this case, so send the stuck count way up to trigger repathing
            self.stuck_count = u8::MAX;
        }
    }
}
