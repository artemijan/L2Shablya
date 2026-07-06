use crate::login_client::LoginClient;
use kameo::prelude::ActorRef;
use l2_core::session::SessionKey;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct GSCharsInfo {
    pub total_chars: u8,
}

#[derive(Debug, Clone, Default)]
pub struct Info {
    pub session: Option<SessionKey>,
    pub player_actor: Option<ActorRef<LoginClient>>,
    pub account_name: String,
    pub is_joined_gs: bool,
    pub chars_on_servers: HashMap<u8, GSCharsInfo>,
    pub game_server: Option<u8>,
    pub gs_id: Option<u8>,
}
