use crate::geoengine::geodata::blocks::IBlock;
use crate::geoengine::geodata::{BLOCK_CELLS, BLOCK_CELLS_X, BLOCK_CELLS_Y};

#[derive(Debug)]
pub struct ComplexBlock {
    data: [i16; BLOCK_CELLS],
}

impl ComplexBlock {
    pub fn new(data: [i16; BLOCK_CELLS]) -> Self {
        Self { data }
    }

    fn get_cell_data(&self, geo_x: i32, geo_y: i32) -> i16 {
        self.data[((geo_x as usize % BLOCK_CELLS_X) * BLOCK_CELLS_Y) + (geo_y as usize % BLOCK_CELLS_Y)]
    }

    fn get_cell_nswe(&self, geo_x: i32, geo_y: i32) -> u8 {
        (self.get_cell_data(geo_x, geo_y) & 0x000F) as u8
    }

    fn get_cell_height(&self, geo_x: i32, geo_y: i32) -> i32 {
        (self.get_cell_data(geo_x, geo_y) & 0xFFF0_u16 as i16) as i32 >> 1
    }
}

impl IBlock for ComplexBlock {
    fn check_nearest_nswe(&self, geo_x: i32, geo_y: i32, _world_z: i32, nswe: u8) -> bool {
        (self.get_cell_nswe(geo_x, geo_y) & nswe) == nswe
    }

    fn get_nearest_z(&self, geo_x: i32, geo_y: i32, _world_z: i32) -> i32 {
        self.get_cell_height(geo_x, geo_y)
    }

    fn get_next_lower_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        let h = self.get_cell_height(geo_x, geo_y);
        if h <= world_z { h } else { world_z }
    }

    fn get_next_higher_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        let h = self.get_cell_height(geo_x, geo_y);
        if h >= world_z { h } else { world_z }
    }
}
