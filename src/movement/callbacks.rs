use screeps::{
    constants::StructureType,
    enums::StructureObject,
    find,
    local::{LocalCostMatrix, RoomName},
    pathfinder::MultiRoomCostResult,
    prelude::*,
};

pub fn callback_standard(room_name: RoomName) -> MultiRoomCostResult {
    let mut new_matrix = LocalCostMatrix::new();
    match screeps::game::rooms().get(room_name) {
        Some(room) => {
            for structure in room.find(find::STRUCTURES, None) {
                let pos = structure.pos();
                match structure {
                    // ignore roads for creeps not needing 'em
                    StructureObject::StructureRoad(_) => {}
                    // containers walkable
                    StructureObject::StructureContainer(_) => {}
                    StructureObject::StructureWall(_) => {
                        new_matrix.set(pos.xy(), 0xff);
                    }
                    StructureObject::StructureRampart(rampart) => {
                        // we could check for and path across public ramparts
                        // (and need to do so if we want to enhance this bot to be able
                        // to cross an ally's public ramparts - but for now, simply don't trust 'em
                        if !rampart.my() {
                            new_matrix.set(pos.xy(), 0xff);
                        }
                    }
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    }
                }
            }

            for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                let pos = csite.pos();
                match csite.structure_type() {
                    // walkable structure types
                    StructureType::Container | StructureType::Road | StructureType::Rampart => {}
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    }
                }
            }
        }
        // can't see the room; terrain matrix is fine
        None => {}
    }
    MultiRoomCostResult::CostMatrix(new_matrix.into())
}

pub fn callback_roads(room_name: RoomName) -> MultiRoomCostResult {
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
                    }
                    // containers walkable
                    StructureObject::StructureContainer(_) => {}
                    StructureObject::StructureWall(_) => {
                        new_matrix.set(pos.xy(), 0xff);
                    }
                    StructureObject::StructureRampart(rampart) => {
                        // we could check for and path across public ramparts
                        // (and need to do so if we want to enhance this bot to be able
                        // to cross an ally's public ramparts - but for now, simply don't trust 'em
                        if !rampart.my() {
                            new_matrix.set(pos.xy(), 0xff);
                        }
                    }
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    }
                }
            }

            for csite in room.find(find::MY_CONSTRUCTION_SITES, None) {
                let pos = csite.pos();
                match csite.structure_type() {
                    // walkable structure types
                    StructureType::Container | StructureType::Road | StructureType::Rampart => {}
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    }
                }
            }
        }
        // can't see the room; terrain matrix is fine
        None => {}
    }
    MultiRoomCostResult::CostMatrix(new_matrix.into())
}

pub fn callback_standard_avoiding_creeps(room_name: RoomName) -> MultiRoomCostResult {
    let mut new_matrix = LocalCostMatrix::new();
    match screeps::game::rooms().get(room_name) {
        Some(room) => {
            for structure in room.find(find::STRUCTURES, None) {
                let pos = structure.pos();
                match structure {
                    // ignore roads for creeps not needing 'em
                    StructureObject::StructureRoad(_) => {}
                    // containers walkable
                    StructureObject::StructureContainer(_) => {}
                    StructureObject::StructureWall(_) => {
                        new_matrix.set(pos.xy(), 0xff);
                    }
                    StructureObject::StructureRampart(rampart) => {
                        // we could check for and path across public ramparts
                        // (and need to do so if we want to enhance this bot to be able
                        // to cross an ally's public ramparts - but for now, simply don't trust 'em
                        if !rampart.my() {
                            new_matrix.set(pos.xy(), 0xff);
                        }
                    }
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    }
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
                    StructureType::Container | StructureType::Road | StructureType::Rampart => {}
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    }
                }
            }
        }
        // can't see the room; terrain matrix is fine
        None => {}
    }
    MultiRoomCostResult::CostMatrix(new_matrix.into())
}

pub fn callback_roads_avoiding_creeps(room_name: RoomName) -> MultiRoomCostResult {
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
                    }
                    // containers walkable
                    StructureObject::StructureContainer(_) => {}
                    StructureObject::StructureWall(_) => {
                        new_matrix.set(pos.xy(), 0xff);
                    }
                    StructureObject::StructureRampart(rampart) => {
                        // we could check for and path across public ramparts
                        // (and need to do so if we want to enhance this bot to be able
                        // to cross an ally's public ramparts - but for now, simply don't trust 'em
                        if !rampart.my() {
                            new_matrix.set(pos.xy(), 0xff);
                        }
                    }
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    }
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
                    StructureType::Container | StructureType::Road | StructureType::Rampart => {}
                    _ => {
                        // other structures, not walkable
                        new_matrix.set(pos.xy(), 0xff);
                    }
                }
            }
        }
        // can't see the room; terrain matrix is fine
        None => {}
    }
    MultiRoomCostResult::CostMatrix(new_matrix.into())
}
