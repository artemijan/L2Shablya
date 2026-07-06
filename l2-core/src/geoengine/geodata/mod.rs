use std::path::Path;
use std::fs::File;
use std::io::{Read, Result};
use crate::geoengine::geodata::regions::Region;
use std::sync::Arc;
use dashmap::DashMap;

pub const GEO_REGIONS_X: usize = 32;
pub const GEO_REGIONS_Y: usize = 32;
pub const WORLD_MIN_X: i32 = -655360;
pub const WORLD_MIN_Y: i32 = -589824;

pub const BLOCK_CELLS_X: usize = 8;
pub const BLOCK_CELLS_Y: usize = 8;
pub const BLOCK_CELLS: usize = BLOCK_CELLS_X * BLOCK_CELLS_Y;

pub const REGION_BLOCKS_X: usize = 256;
pub const REGION_BLOCKS_Y: usize = 256;
pub const REGION_BLOCKS: usize = REGION_BLOCKS_X * REGION_BLOCKS_Y;

pub const REGION_CELLS_X: usize = REGION_BLOCKS_X * BLOCK_CELLS_X;
pub const REGION_CELLS_Y: usize = REGION_BLOCKS_Y * BLOCK_CELLS_Y;
pub mod regions;
pub mod blocks;

#[derive(Debug)]
pub struct GeoData {
    regions: DashMap<usize, Arc<Region>>,
}

impl GeoData {
    pub fn new() -> Self {
        Self {
            regions: DashMap::new(),
        }
    }

    pub fn load_region(&self, file_path: &Path, region_x: i32, region_y: i32) -> Result<()> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let region = Region::new(&buffer);
        let region_offset = (region_x as usize * GEO_REGIONS_Y) + region_y as usize;
        self.regions.insert(region_offset, Arc::new(region));
        Ok(())
    }

    pub fn get_region(&self, geo_x: i32, geo_y: i32) -> Option<Arc<Region>> {
        let region_x = geo_x / crate::geoengine::geodata::REGION_CELLS_X as i32;
        let region_y = geo_y / crate::geoengine::geodata::REGION_CELLS_Y as i32;
        let region_offset = (region_x as usize * GEO_REGIONS_Y) + region_y as usize;
        self.regions.get(&region_offset).map(|r| Arc::clone(r.value()))
    }

    pub fn get_geo_x(&self, world_x: i32) -> i32 {
        (world_x - WORLD_MIN_X) / 16
    }

    pub fn get_geo_y(&self, world_y: i32) -> i32 {
        (world_y - WORLD_MIN_Y) / 16
    }

    pub fn get_world_x(&self, geo_x: i32) -> i32 {
        (geo_x * 16) + WORLD_MIN_X + 8
    }

    pub fn get_world_y(&self, geo_y: i32) -> i32 {
        (geo_y * 16) + WORLD_MIN_Y + 8
    }

    pub fn get_nearest_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        if let Some(region) = self.get_region(geo_x, geo_y) {
            region.get_nearest_z(geo_x, geo_y, world_z)
        } else {
            world_z
        }
    }

    pub fn get_next_lower_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        if let Some(region) = self.get_region(geo_x, geo_y) {
            region.get_next_lower_z(geo_x, geo_y, world_z)
        } else {
            world_z
        }
    }

    pub fn get_next_higher_z(&self, geo_x: i32, geo_y: i32, world_z: i32) -> i32 {
        if let Some(region) = self.get_region(geo_x, geo_y) {
            region.get_next_higher_z(geo_x, geo_y, world_z)
        } else {
            world_z
        }
    }

    pub fn check_nearest_nswe(&self, geo_x: i32, geo_y: i32, world_z: i32, nswe: u8) -> bool {
        if let Some(region) = self.get_region(geo_x, geo_y) {
            region.check_nearest_nswe(geo_x, geo_y, world_z, nswe)
        } else {
            true
        }
    }

    pub fn has_geo_pos(&self, geo_x: i32, geo_y: i32) -> bool {
        self.get_region(geo_x, geo_y).is_some()
    }
}
