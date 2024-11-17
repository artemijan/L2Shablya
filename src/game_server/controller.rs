use crate::common::traits::IpBan;
use crate::game_server::dto::config::GSServer;
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Controller {
    cfg: Arc<GSServer>,
}

impl Controller {
    pub fn new(cfg: Arc<GSServer>) -> Self {
        Controller { cfg }
    }
}

impl IpBan for Controller {
    fn is_ip_banned(&self, _: &str) -> bool {
        false
    }
}
