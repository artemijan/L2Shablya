use std::net::IpAddr;

use num::{BigInt, Num};

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
    pub servers: Vec<GSCharsInfo>,
}

impl Info {
    pub fn new()->Self{
        Info{
            is_authed: false,
            ..Info::default()
        }
    }
}
