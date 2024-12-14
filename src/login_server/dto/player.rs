use crate::common::session::SessionKey;
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, Default)]
pub struct GSCharsInfo {
    pub total_chars: u8,
    pub chars_to_delete: u8,
    pub char_deletion_timestamps: Vec<i64>,
}
#[derive(Debug, Clone, Default)]
pub struct Info {
    pub session: Option<SessionKey>,
    pub account_name: String,
    pub login_time: Option<bool>,
    pub is_authed: bool,
    pub is_joined_gs: bool,
    pub ip_address: Option<IpAddr>,
    pub chars_on_servers: HashMap<u8, GSCharsInfo>,
    pub game_server: Option<u8>,
}

impl Info {
    pub fn new() -> Self {
        Info {
            is_authed: false,
            is_joined_gs: false,
            ..Info::default()
        }
    }
}
