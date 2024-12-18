use l2_core::config::gs::GSServer;
use l2_core::traits::IpBan;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Controller {
    cfg: Arc<GSServer>,
    online_accounts: Vec<String>,
}

impl Controller {
    pub fn new(cfg: Arc<GSServer>) -> Self {
        Controller {
            cfg,
            online_accounts: Vec::new(),
        }
    }
    pub fn get_cfg(&self) -> Arc<GSServer> {
        self.cfg.clone()
    }
    
    pub fn get_online_accounts(&self) -> &Vec<String> {
        &self.online_accounts
    }
}

impl IpBan for Controller {
    fn is_ip_banned(&self, _: &str) -> bool {
        false
    }
}
