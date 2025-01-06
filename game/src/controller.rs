use crate::data::exp_table::ExpTable;
use dashmap::DashMap;
use l2_core::config::gs::GSServer;
use l2_core::dto::Player;
use l2_core::message_broker::MessageBroker;
use l2_core::packets::common::PacketType;
use l2_core::traits::IpBan;
use std::sync::Arc;
use std::time::Duration;
use entities::entities::prelude::Character;

#[derive(Clone, Debug)]
pub struct Controller {
    cfg: Arc<GSServer>,
    pub exp_table: ExpTable,
    online_accounts: DashMap<String, Player>,
    pub hero_list:DashMap<i32,Character>,
    pub message_broker: Arc<MessageBroker<u8, PacketType>>,
}

impl Controller {
    pub fn new(cfg: Arc<GSServer>) -> Self {
        let threshold = Duration::from_secs(u64::from(cfg.listeners.login_server.messages.timeout));
        let exp_table = ExpTable::load();
        Controller {
            exp_table,
            cfg,
            hero_list:DashMap::new(),
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
    pub fn remove_online_account(&self, account: &str) {
        self.online_accounts.remove(account);
    }
}

impl IpBan for Controller {
    fn is_ip_banned(&self, _: &str) -> bool {
        false
    }
}
