use std::net::IpAddr;

use num::{BigInt, Num};

use crate::common::session::SessionKey;

#[derive(Debug, Clone, Default)]
pub struct Info {
    pub session: Option<SessionKey>,
    pub account_name: Option<String>,
    pub login_time: Option<bool>,
    pub is_authed: bool,
    pub ip_address: Option<IpAddr>,
    pub servers: Option<Vec<String>>,
}
