use dashmap::DashMap;
use l2_core::config::gs::GSServer;
use l2_core::dto::Player;
use l2_core::message_broker::MessageBroker;
use l2_core::packets::common::PacketType;
use l2_core::traits::IpBan;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Controller {
    cfg: Arc<GSServer>,
    online_accounts: DashMap<String, Player>,
    pub message_broker: Arc<MessageBroker<u8, PacketType>>,
}

impl Controller {
    pub fn new(cfg: Arc<GSServer>) -> Self {
        let threshold = Duration::from_secs(u64::from(cfg.listeners.login_server.messages.timeout));
        Controller {
            cfg,
            message_broker: MessageBroker::new(threshold),
            online_accounts: DashMap::new(),
        }
    }
    pub fn get_cfg(&self) -> Arc<GSServer> {
        self.cfg.clone()
    }

    pub fn get_online_accounts(&self) -> Vec<String> {
        self.online_accounts
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    pub fn add_online_account(&self, account: String) -> Option<Player> {
        let key = account.clone();
        self.online_accounts.insert(
            key,
            Player {
                login_name: account,
            },
        )
    }
}

impl IpBan for Controller {
    fn is_ip_banned(&self, _: &str) -> bool {
        false
    }
}
