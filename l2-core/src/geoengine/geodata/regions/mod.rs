use crate::geoengine::geodata::blocks::complex_block::ComplexBlock;
use crate::geoengine::geodata::blocks::flat_block::FlatBlock;
use crate::geoengine::geodata::blocks::multilayer_block::MultilayerBlock;
use crate::geoengine::geodata::blocks::IBlock;
use crate::geoengine::geodata::{BLOCK_CELLS, BLOCK_CELLS_X, BLOCK_CELLS_Y, REGION_BLOCKS, REGION_BLOCKS_X, REGION_BLOCKS_Y};

#[derive(Debug)]
pub struct Region {
    blocks: Vec<Box<dyn IBlock>>,
}

impl Region {
    pub fn new(buffer: &[u8]) -> Self {
        let mut blocks: Vec<Box<dyn IBlock>> = Vec::with_capacity(REGION_BLOCKS);
        let mut offset = 0;

        for _ in 0..REGION_BLOCKS {
            let block_type = buffer[offset];
            offset += 1;

            match block_type {
                0 => {
                    // Flat
                    let height = i16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
                    offset += 2;
                    blocks.push(Box::new(FlatBlock::new(height)));
                }
                1 => {
                    // Complex
                    let mut data = [0i16; BLOCK_CELLS];
                    for i in 0..BLOCK_CELLS {
                        data[i] = i16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
                        offset += 2;
                    }
                    blocks.push(Box::new(ComplexBlock::new(data)));
                }
                2 => {
                    // Multilayer
                    let start_offset = offset;
                    for _ in 0..BLOCK_CELLS {
                        let n_layers = buffer[offset];
                        offset += 1 + (n_layers as usize * 2);
                    }
                    let block_data = buffer[start_offset..offset].to_vec();
                    blocks.push(Box::new(MultilayerBlock::new(block_data)));
                }
                _ => panic!("Invalid block type {}", block_type),
            }
        }

        Self { blocks }
    }

    fn get_block(&self, geo_x: i32, geo_y: i32) -> &dyn IBlock {
        let block_index = (((geo_x as usize / BLOCK_CELLS_X) % REGION_BLOCKS_X) * REGION_BLOCKS_Y)
            + ((geo_y as usize / BLOCK_CELLS_Y) % REGION_BLOCKS_Y);
        self.blocks[block_index].as_ref()
    }

    pub fn check_nearest_nswe(&self, geo_x: i32, geo_y: i32, world_z: i32, nswe: u8) -> bool {
        self.get_block(geo_x, geo_y).check_nearest_nswe(geo_x, geo_y, world_z, nswe)
    }

    pub fn get_nearest_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        self.get_block(geo_x, geo_y).get_nearest_z(geo_x, geo_y, world_z)
    }

    pub fn get_next_lower_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        self.get_block(geo_x, geo_y).get_next_lower_z(geo_x, geo_y, world_z)
    }

    pub fn get_next_higher_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        self.get_block(geo_x, geo_y).get_next_higher_z(geo_x, geo_y, world_z)
    }
}
