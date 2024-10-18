use num::{BigInt, Num};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct GSInfo {
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

impl GSInfo {
    pub fn hex(&self) -> String {
        BigInt::from_signed_bytes_be(&self.hex_id).to_str_radix(16)
    }
}
