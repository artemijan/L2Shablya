#[derive(Debug, Clone)]
pub struct GameServerInfo {
    pub id: u8,
    pub accept_alternative_id: bool,
    pub host_reserved: bool,
    pub port: u16,
    pub is_authed: bool,
    pub status: i32,
    pub is_pvp: bool,
    pub server_type: i32,
    pub age_limit: u8,
    pub show_brackets: bool,
    pub max_players: u32,
    pub hex_id: Vec<u8>,
    pub hosts: Vec<String>,
}