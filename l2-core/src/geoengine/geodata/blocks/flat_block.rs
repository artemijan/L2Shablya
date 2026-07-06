use crate::geoengine::geodata::blocks::IBlock;

#[derive(Debug)]
pub struct FlatBlock {
    height: i16,
}

impl FlatBlock {
    pub fn new(height: i16) -> Self {
        Self { height }
    }
}

impl IBlock for FlatBlock {
    fn check_nearest_nswe(&self, _geo_x: i32, _geo_y: i32, _world_z: i32, _nswe: u8) -> bool {
        true
    }

    fn get_nearest_z(&self, _geo_x: i32, _geo_y: i32, _world_z: i32) -> i32 {
        self.height as i32
    }

    fn get_next_lower_z(&self, _geo_x: i32, _geo_y: i32, world_z: i32) -> i32 {
        if self.height as i32 <= world_z {
            self.height as i32
        } else {
            world_z
        }
    }

    fn get_next_higher_z(&self, _geo_x: i32, _geo_y: i32, world_z: i32) -> i32 {
        if self.height as i32 >= world_z {
            self.height as i32
        } else {
            world_z
        }
    }
}
