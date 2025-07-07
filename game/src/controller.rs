use l2_core::data::action_list::ActionList;
use l2_core::data::base_stat::BaseStat;
use l2_core::data::char_template::ClassTemplates;
use l2_core::data::exp_table::ExpTable;
use crate::ls_client::LoginServerClient;
use crate::managers::ClanAllyManager;
use anyhow::anyhow;
use dashmap::DashMap;
use entities::entities::character;
use entities::DBPool;
use kameo::actor::ActorRef;
use l2_core::config::gs::GSServerConfig;
use l2_core::config::traits::{ConfigDirLoader, ConfigFileLoader};
use l2_core::traits::IpBan;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[derive(Clone, Debug)]
pub struct GameController {
    cfg: Arc<GSServerConfig>,
    pub exp_table: ExpTable,
    pub action_list: ActionList,
    pub class_templates: Arc<ClassTemplates>,
    ls_actor: Arc<RwLock<Option<ActorRef<LoginServerClient>>>>,
    online_accounts: DashMap<String, String>,
    pub base_stats_table: BaseStat,
    pub hero_list: DashMap<i32, character::Model>,
    pub clan_ally_manager: Arc<RwLock<ClanAllyManager>>,
}

impl GameController {
    pub async fn new(cfg: Arc<GSServerConfig>, db_pool: &DBPool) -> Self {
        let exp_table = ExpTable::load();
        let action_list = ActionList::load();
        let class_templates = ClassTemplates::load();
        let base_stats = BaseStat::load();
        GameController {
            exp_table,
            cfg,
            ls_actor: Arc::new(RwLock::new(None)),
            action_list,
            base_stats_table: base_stats,
            class_templates: Arc::new(class_templates),
            hero_list: DashMap::new(),
            online_accounts: DashMap::new(),
            clan_ally_manager: Arc::new(RwLock::new(ClanAllyManager::new(db_pool.clone()).await)),
        }
    }
    pub async fn set_ls_actor(&self, actor: ActorRef<LoginServerClient>) {
        *self.ls_actor.write().await = Some(actor);
    }

    pub async fn get_ls_actor(&self) -> Option<ActorRef<LoginServerClient>> {
        self.ls_actor.read().await.clone()
    }
    pub async fn try_get_ls_actor(&self) -> anyhow::Result<ActorRef<LoginServerClient>> {
        self.ls_actor
            .read()
            .await
            .clone()
            .ok_or_else(|| anyhow!("LS actor not found"))
    }
    
    pub fn get_cfg(&self) -> Arc<GSServerConfig> {
        self.cfg.clone()
    }
    #[allow(clippy::unused_self)]
    pub fn get_game_time(&self) -> i32 {
        //todo time player spend in game 
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
impl GameController {
    pub fn from_config(cfg: Arc<GSServerConfig>) -> Self {
        let exp_table = ExpTable::load();
        let action_list = ActionList::load();
        let class_templates = ClassTemplates::load();
        let base_stats = BaseStat::load();
        GameController {
            exp_table,
            cfg,
            action_list,
            ls_actor: Arc::new(RwLock::new(None)),
            base_stats_table: base_stats,
            class_templates: Arc::new(class_templates),
            hero_list: DashMap::new(),
            online_accounts: DashMap::new(),
            clan_ally_manager: Arc::new(RwLock::new(ClanAllyManager::default())),
        }
    }
}
impl IpBan for GameController {
    fn is_ip_banned(&self, _: &str) -> bool {
        false
    }
}
