#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
}
impl Location {
    pub const TILE_SIZE: i32 = 32768;
    pub const TILE_X_MIN: i32 = 11;
    pub const TILE_Y_MIN: i32 = 10;
    pub const TILE_X_MAX: i32 = 28;
    pub const TILE_Y_MAX: i32 = 26;
    pub const TILE_ZERO_COORD_X: i32 = 20;
    pub const TILE_ZERO_COORD_Y: i32 = 18;
    pub const WORLD_X_MIN: i32 = (Self::TILE_X_MIN - Self::TILE_ZERO_COORD_X) * Self::TILE_SIZE;
    pub const WORLD_Y_MIN: i32 = (Self::TILE_Y_MIN - Self::TILE_ZERO_COORD_Y) * Self::TILE_SIZE;

    pub const WORLD_X_MAX: i32 =
        ((Self::TILE_X_MAX - Self::TILE_ZERO_COORD_X) + 1) * Self::TILE_SIZE;
    pub const WORLD_Y_MAX: i32 =
        ((Self::TILE_Y_MAX - Self::TILE_ZERO_COORD_Y) + 1) * Self::TILE_SIZE;

    pub fn spawn(&mut self, mut x: i32, mut y: i32, z: i32) {
        if x < Self::WORLD_X_MIN {
            x = Self::WORLD_X_MIN + 5000;
        } else if x > Self::WORLD_X_MAX {
            x = Self::WORLD_X_MAX - 5000;
        }
        if y < Self::WORLD_Y_MIN {
            y = Self::WORLD_Y_MIN + 5000;
        } else if y > Self::WORLD_Y_MAX {
            y = Self::WORLD_Y_MAX - 5000;
        }
        self.x = x;
        self.y = y;
        self.z = z;
    }
}
#[derive(Debug, Clone, Copy)]
pub enum ZoneId {
    Pvp,
    Peace,
    Siege,
    MotherTree,
    ClanHall,
    Landing,
    NoLanding,
    Water,
    Jail,
    MonsterTrack,
    Castle,
    Swamp,
    NoSummonFriend,
    Fort,
    NoStore,
    NoPvp,
    Script,
    Hq,
    DangerArea,
    Altered,
    NoBookmark,
    NoItemDrop,
    NoRestart,
    Sayune,
    Fishing,
    Undying,
    Tax,
}
