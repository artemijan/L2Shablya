pub enum BlockType {
    Flat = 0,
    Complex = 1,
    Multilayer = 2,
}

pub trait IBlock: Send + Sync + std::fmt::Debug {
    fn check_nearest_nswe(&self, geo_x: i32, geo_y: i32, world_z: i32, nswe: u8) -> bool;
    fn get_nearest_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32;
    fn get_next_lower_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32;
    fn get_next_higher_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32;
}

pub mod flat_block;
pub mod complex_block;
pub mod multilayer_block;
