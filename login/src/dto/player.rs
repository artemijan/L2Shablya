use l2_core::session::SessionKey;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use kameo::prelude::ActorRef;
use crate::login_client::LoginClient;

#[derive(Debug, Clone, Default)]
pub struct GSCharsInfo {
    pub total_chars: u8,
    pub chars_to_delete: u8,
    pub char_deletion_timestamps: Vec<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct Info {
    pub session: Option<SessionKey>,
    pub player_actor: Option<ActorRef<LoginClient>>,
    pub account_name: String,
    pub ip_address: Option<Ipv4Addr>,
    pub is_joined_gs: bool,
    pub is_authed: bool,
    pub chars_on_servers: HashMap<u8, GSCharsInfo>,
    pub game_server: Option<u8>,
}
