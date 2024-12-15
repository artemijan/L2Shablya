use crate::common::traits::IpBan;
use std::sync::Arc;
use crate::common::config::gs::GSServer;

#[derive(Clone, Debug)]
pub struct Controller {
    cfg: Arc<GSServer>,
}

impl Controller {
    pub fn new(cfg: Arc<GSServer>) -> Self {
        Controller { cfg }
    }
    pub fn get_cfg(&self) -> Arc<GSServer> {
        self.cfg.clone()
    }
    pub fn get_online_accounts(&self) -> Vec<String> {
        // todo
        vec![]
    }
}

impl IpBan for Controller {
    fn is_ip_banned(&self, _: &str) -> bool {
        false
    }
}
