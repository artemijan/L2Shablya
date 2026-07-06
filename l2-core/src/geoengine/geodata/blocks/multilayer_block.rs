use crate::geoengine::geodata::blocks::IBlock;
use crate::geoengine::geodata::{BLOCK_CELLS_X, BLOCK_CELLS_Y};

#[derive(Debug)]
pub struct MultilayerBlock {
    data: Vec<u8>,
}

impl MultilayerBlock {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    fn get_cell_data_offset(&self, geo_x: i32, geo_y: i32) -> usize {
        let cell_local_offset = ((geo_x as usize % BLOCK_CELLS_X) * BLOCK_CELLS_Y) + (geo_y as usize % BLOCK_CELLS_Y);
        let mut cell_data_offset = 0;
        for _ in 0..cell_local_offset {
            let n_layers = self.data[cell_data_offset];
            cell_data_offset += 1 + (n_layers as usize * 2);
        }
        cell_data_offset
    }

    fn extract_layer_data(&self, data_offset: usize) -> i16 {
        (self.data[data_offset] as u16 | ((self.data[data_offset + 1] as u16) << 8)) as i16
    }

    fn extract_layer_nswe(&self, layer_data: i16) -> u8 {
        (layer_data & 0x000F) as u8
    }

    fn extract_layer_height(&self, layer_data: i16) -> i32 {
        (layer_data & 0xFFF0_u16 as i16) as i32 >> 1
    }
}

impl IBlock for MultilayerBlock {
    fn check_nearest_nswe(&self, geo_x: i32, geo_y: i32, world_z: i32, nswe: u8) -> bool {
        let start_offset = self.get_cell_data_offset(geo_x, geo_y);
        let n_layers = self.data[start_offset];
        let mut nearest_dz = i32::MAX;
        let mut nearest_nswe = 0;

        for i in 0..n_layers as usize {
            let offset = start_offset + 1 + (i * 2);
            let layer_data = self.extract_layer_data(offset);
            let layer_z = self.extract_layer_height(layer_data);
            if layer_z == world_z {
                return (self.extract_layer_nswe(layer_data) & nswe) == nswe;
            }
            let dz = (layer_z - world_z).abs();
            if dz < nearest_dz {
                nearest_dz = dz;
                nearest_nswe = self.extract_layer_nswe(layer_data);
            }
        }
        (nearest_nswe & nswe) == nswe
    }

    fn get_nearest_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        let start_offset = self.get_cell_data_offset(geo_x, geo_y);
        let n_layers = self.data[start_offset];
        let mut nearest_dz = i32::MAX;
        let mut nearest_z = world_z;

        for i in 0..n_layers as usize {
            let offset = start_offset + 1 + (i * 2);
            let layer_data = self.extract_layer_data(offset);
            let layer_z = self.extract_layer_height(layer_data);
            if layer_z == world_z {
                return layer_z;
            }
            let dz = (layer_z - world_z).abs();
            if dz < nearest_dz {
                nearest_dz = dz;
                nearest_z = layer_z;
            }
        }
        nearest_z
    }

    fn get_next_lower_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        let start_offset = self.get_cell_data_offset(geo_x, geo_y);
        let n_layers = self.data[start_offset];
        let mut lower_z = i32::MIN;

        for i in 0..n_layers as usize {
            let offset = start_offset + 1 + (i * 2);
            let layer_data = self.extract_layer_data(offset);
            let layer_z = self.extract_layer_height(layer_data);
            if layer_z == world_z {
                return layer_z;
            }
            if layer_z < world_z && layer_z > lower_z {
                lower_z = layer_z;
            }
        }
        if lower_z == i32::MIN { world_z } else { lower_z }
    }

    fn get_next_higher_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        let start_offset = self.get_cell_data_offset(geo_x, geo_y);
        let n_layers = self.data[start_offset];
        let mut higher_z = i32::MAX;

        for i in 0..n_layers as usize {
            let offset = start_offset + 1 + (i * 2);
            let layer_data = self.extract_layer_data(offset);
            let layer_z = self.extract_layer_height(layer_data);
            if layer_z == world_z {
                return layer_z;
            }
            if layer_z > world_z && layer_z < higher_z {
                higher_z = layer_z;
            }
        }
        if higher_z == i32::MAX { world_z } else { higher_z }
    }
}
