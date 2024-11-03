use std::collections::HashMap;
use std::net::IpAddr;
use crate::common::session::SessionKey;

#[derive(Debug, Clone, Default)]
pub struct GSCharsInfo {
    pub chars: u8,
    pub chars_to_delete: u8,
    pub char_list: Vec<i64>,
}
#[derive(Debug, Clone, Default)]
pub struct Info {
    pub session: Option<SessionKey>,
    pub account_name: String,
    pub login_time: Option<bool>,
    pub is_authed: bool,
    pub ip_address: Option<IpAddr>,
    pub chars_on_servers: HashMap<u8, GSCharsInfo>,
    pub game_server: Option<u8>
}

impl Info {
    pub fn new() -> Self {
        Info {
            is_authed: false,
            ..Info::default()
        }
    }
}
