use crate::data::base_stat::BaseStat;
use crate::data::char_template::ClassTemplates;
use crate::data::exp_table::ExpTable;
use crate::managers::ClanAllyManager;
use dashmap::DashMap;
use entities::entities::character;
use entities::DBPool;
use l2_core::config::gs::GSServer;
use l2_core::config::traits::{ConfigDirLoader, ConfigFileLoader};
use l2_core::message_broker::MessageBroker;
use l2_core::shared_packets::common::PacketType;
use l2_core::traits::IpBan;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Controller {
    cfg: Arc<GSServer>,
    pub exp_table: ExpTable,
    pub class_templates: Arc<ClassTemplates>,
    online_accounts: DashMap<String, String>,
    pub base_stats_table: BaseStat,
    pub hero_list: DashMap<i32, character::Model>,
    pub clan_ally_manager: Arc<RwLock<ClanAllyManager>>,
    pub message_broker: Arc<MessageBroker<u8, PacketType>>,
}

impl Controller {
    pub async fn new(cfg: Arc<GSServer>, db_pool: &DBPool) -> Self {
        let threshold = Duration::from_secs(u64::from(cfg.listeners.login_server.messages.timeout));
        let exp_table = ExpTable::load();
        let class_templates = ClassTemplates::load();
        let base_stats = BaseStat::load();
        Controller {
            exp_table,
            cfg,
            base_stats_table: base_stats,
            class_templates: Arc::new(class_templates),
            hero_list: DashMap::new(),
            message_broker: MessageBroker::new(threshold),
            online_accounts: DashMap::new(),
            clan_ally_manager: Arc::new(RwLock::new(ClanAllyManager::new(db_pool.clone()).await)),
        }
    }

    pub fn get_cfg(&self) -> Arc<GSServer> {
        self.cfg.clone()
    }
    #[allow(clippy::unused_self)]
    pub fn get_game_time(&self) -> i32 {
        //todo game time
        9999
    }
    pub fn get_online_accounts(&self) -> Vec<String> {
        self.online_accounts
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    pub fn add_online_account(&self, account: String) -> Option<String> {
        let key = account.clone();
        self.online_accounts.insert(key, account)
    }
    pub fn logout_account(&self, account: &str) {
        self.online_accounts.remove(account);
        info!("Logged out online account: {}", account);
    }
}

#[cfg(test)]
impl Controller {
    pub fn from_config(cfg: Arc<GSServer>) -> Self {
        let threshold = Duration::from_secs(u64::from(cfg.listeners.login_server.messages.timeout));
        let exp_table = ExpTable::load();
        let class_templates = ClassTemplates::load();
        let base_stats = BaseStat::load();
        Controller {
            exp_table,
            cfg,
            base_stats_table: base_stats,
            class_templates: Arc::new(class_templates),
            hero_list: DashMap::new(),
            message_broker: MessageBroker::new(threshold),
            online_accounts: DashMap::new(),
            clan_ally_manager: Arc::new(RwLock::new(ClanAllyManager::default())),
        }
    }
}
impl IpBan for Controller {
    fn is_ip_banned(&self, _: &str) -> bool {
        false
    }
}
