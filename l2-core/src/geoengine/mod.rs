use crate::geoengine::geodata::GeoData;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;
use tracing::{info, warn};

pub mod geodata;

pub const NSWE_EAST: u8 = 1 << 0;
pub const NSWE_WEST: u8 = 1 << 1;
pub const NSWE_SOUTH: u8 = 1 << 2;
pub const NSWE_NORTH: u8 = 1 << 3;

pub const NSWE_NORTH_EAST: u8 = NSWE_NORTH | NSWE_EAST;
pub const NSWE_NORTH_WEST: u8 = NSWE_NORTH | NSWE_WEST;
pub const NSWE_SOUTH_EAST: u8 = NSWE_SOUTH | NSWE_EAST;
pub const NSWE_SOUTH_WEST: u8 = NSWE_SOUTH | NSWE_WEST;

pub const NSWE_ALL: u8 = NSWE_EAST | NSWE_WEST | NSWE_SOUTH | NSWE_NORTH;

#[derive(Debug)]
pub struct GeoEngine {
    geodata: Arc<GeoData>,
}

impl GeoEngine {
    pub fn new(geo_path: &Path) -> Self {
        let geodata = Arc::new(GeoData::new());
        let mut loaded_regions = 0;

        info!("Loading geodata from {:?}", geo_path);

        for entry in WalkDir::new(geo_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("l2j") {
                if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                    let parts: Vec<&str> = file_name.split('_').collect();
                    if parts.len() == 2 {
                        if let (Ok(rx), Ok(ry)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                            match geodata.load_region(path, rx, ry) {
                                Ok(_) => loaded_regions += 1,
                                Err(e) => warn!("Failed to load geodata region {:?}: {}", path, e),
                            }
                        }
                    }
                }
            }
        }

        info!("Loaded {} geodata regions", loaded_regions);

        Self { geodata }
    }

    pub fn get_nearest_z(&self, x: i32, y: i32, z: i32) -> i32 {
        let geo_x = self.geodata.get_geo_x(x);
        let geo_y = self.geodata.get_geo_y(y);
        self.geodata.get_nearest_z(geo_x, geo_y, z)
    }

    pub fn check_nearest_nswe(&self, geo_x: i32, geo_y: i32, world_z: i32, nswe: u8) -> bool {
        self.geodata.check_nearest_nswe(geo_x, geo_y, world_z, nswe)
    }

    pub fn check_nearest_nswe_anti_corner_cut(&self, geo_x: i32, geo_y: i32, world_z: i32, nswe: u8) -> bool {
        let mut can = true;
        if (nswe & NSWE_NORTH_EAST) == NSWE_NORTH_EAST {
            can = self.check_nearest_nswe(geo_x, geo_y - 1, world_z, NSWE_EAST) && self.check_nearest_nswe(geo_x + 1, geo_y, world_z, NSWE_NORTH);
        }

        if can && (nswe & NSWE_NORTH_WEST) == NSWE_NORTH_WEST {
            can = self.check_nearest_nswe(geo_x, geo_y - 1, world_z, NSWE_WEST) && self.check_nearest_nswe(geo_x - 1, geo_y, world_z, NSWE_NORTH);
        }

        if can && (nswe & NSWE_SOUTH_EAST) == NSWE_SOUTH_EAST {
            can = self.check_nearest_nswe(geo_x, geo_y + 1, world_z, NSWE_EAST) && self.check_nearest_nswe(geo_x + 1, geo_y, world_z, NSWE_SOUTH);
        }

        if can && (nswe & NSWE_SOUTH_WEST) == NSWE_SOUTH_WEST {
            can = self.check_nearest_nswe(geo_x, geo_y + 1, world_z, NSWE_WEST) && self.check_nearest_nswe(geo_x - 1, geo_y, world_z, NSWE_SOUTH);
        }

        can && self.check_nearest_nswe(geo_x, geo_y, world_z, nswe)
    }

    pub fn can_move(&self, x: i32, y: i32, z: i32, tx: i32, ty: i32, tz: i32) -> bool {
        let geo_x = self.geodata.get_geo_x(x);
        let geo_y = self.geodata.get_geo_y(y);
        let t_geo_x = self.geodata.get_geo_x(tx);
        let t_geo_y = self.geodata.get_geo_y(ty);

        if geo_x == t_geo_x && geo_y == t_geo_y {
            return true;
        }

        let point_iter = LinePointIterator::new(geo_x, geo_y, t_geo_x, t_geo_y);
        let mut prev_x = geo_x;
        let mut prev_y = geo_y;
        let mut prev_z = self.geodata.get_nearest_z(geo_x, geo_y, z);

        for (cur_x, cur_y) in point_iter {
            let cur_z = self.geodata.get_nearest_z(cur_x, cur_y, prev_z);
            let nswe = compute_nswe(prev_x, prev_y, cur_x, cur_y);
            if !self.check_nearest_nswe_anti_corner_cut(prev_x, prev_y, prev_z, nswe) {
                return false;
            }
            prev_x = cur_x;
            prev_y = cur_y;
            prev_z = cur_z;
        }

        let target_z = self.geodata.get_nearest_z(t_geo_x, t_geo_y, tz);
        prev_z == target_z
    }

    pub fn can_see(&self, x: i32, y: i32, z: i32, tx: i32, ty: i32, tz: i32) -> bool {
        let geo_x = self.geodata.get_geo_x(x);
        let geo_y = self.geodata.get_geo_y(y);
        let t_geo_x = self.geodata.get_geo_x(tx);
        let t_geo_y = self.geodata.get_geo_y(ty);

        let nearest_from_z = self.geodata.get_nearest_z(geo_x, geo_y, z);
        let nearest_to_z = self.geodata.get_nearest_z(t_geo_x, t_geo_y, tz);

        if geo_x == t_geo_x && geo_y == t_geo_y {
            return true;
        }

        let point_iter = LinePointIterator3D::new(geo_x, geo_y, nearest_from_z, t_geo_x, t_geo_y, nearest_to_z);
        let mut prev_x = geo_x;
        let mut prev_y = geo_y;
        let mut prev_z = nearest_from_z;

        for (cur_x, cur_y, bee_cur_z) in point_iter.skip(1) {
            if cur_x == prev_x && cur_y == prev_y {
                continue;
            }

            let nswe = compute_nswe(prev_x, prev_y, cur_x, cur_y);
            let cur_geo_z = if self.geodata.check_nearest_nswe(prev_x, prev_y, prev_z, nswe) {
                self.geodata.get_nearest_z(cur_x, cur_y, prev_z)
            } else {
                self.geodata.get_next_higher_z(cur_x, cur_y, prev_z)
            };

            if cur_geo_z > bee_cur_z + 48 {
                return false;
            }

            prev_x = cur_x;
            prev_y = cur_y;
            prev_z = cur_geo_z;
        }

        true
    }
}

pub fn compute_nswe(x: i32, y: i32, tx: i32, ty: i32) -> u8 {
    let mut nswe = 0;
    if tx > x {
        if ty > y {
            // South-East
            // In many L2 geo engines, moving diagonally requires both directions to be open,
            // or sometimes a specific diagonal flag. Looking at Mobius' GeoEngine.java:
            // It uses target NSWE for diagonal checks.
        }
        nswe |= NSWE_EAST;
    } else if tx < x {
        nswe |= NSWE_WEST;
    }
    if ty > y {
        nswe |= NSWE_SOUTH;
    } else if ty < y {
        nswe |= NSWE_NORTH;
    }
    nswe
}

pub struct LinePointIterator {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    err: i32,
    started: bool,
    finished: bool,
}

impl LinePointIterator {
    pub fn new(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let err = dx - dy;
        Self { x1, y1, x2, y2, dx, dy, sx, sy, err, started: false, finished: false }
    }
}

impl Iterator for LinePointIterator {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        if !self.started {
            self.started = true;
            if self.x1 == self.x2 && self.y1 == self.y2 {
                self.finished = true;
            }
            return Some((self.x1, self.y1));
        }
        if self.x1 == self.x2 && self.y1 == self.y2 {
            self.finished = true;
            return None;
        }
        let e2 = 2 * self.err;
        if e2 > -self.dy {
            self.err -= self.dy;
            self.x1 += self.sx;
        }
        if e2 < self.dx {
            self.err += self.dx;
            self.y1 += self.sy;
        }
        Some((self.x1, self.y1))
    }
}

pub struct LinePointIterator3D {
    x_iter: LinePointIterator,
    z1: i32,
    z2: i32,
    steps: i32,
    current_step: i32,
}

impl LinePointIterator3D {
    pub fn new(x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) -> Self {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let steps = std::cmp::max(dx, dy);
        Self {
            x_iter: LinePointIterator::new(x1, y1, x2, y2),
            z1,
            z2,
            steps,
            current_step: 0,
        }
    }
}

impl Iterator for LinePointIterator3D {
    type Item = (i32, i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((x, y)) = self.x_iter.next() {
            let z = if self.steps == 0 {
                self.z1
            } else {
                self.z1 + (self.z2 - self.z1) * self.current_step / self.steps
            };
            self.current_step += 1;
            Some((x, y, z))
        } else {
            None
        }
    }
}
