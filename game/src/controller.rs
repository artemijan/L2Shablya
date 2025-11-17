use crate::ls_client::LoginServerClient;
use crate::managers::ClanAllyManager;
use crate::packets::to_client::CharInfo;
use crate::pl_client::{GetCharInfo, PlayerClient};
use anyhow::anyhow;
use dashmap::DashMap;
use entities::entities::character;
use entities::DBPool;
use kameo::actor::ActorRef;
use l2_core::config::gs::GSServerConfig;
use l2_core::config::traits::{ConfigDirLoader, ConfigFileLoader};
use l2_core::data::action_list::ActionList;
use l2_core::data::base_stat::BaseStat;
use l2_core::data::char_template::ClassTemplates;
use l2_core::data::exp_table::ExpTable;
use l2_core::data::SkillTreesData;
use l2_core::game_objects::player::Player;
use l2_core::network::connection::HandleOutboundPacket;
use l2_core::shared_packets::common::SendablePacket;
use l2_core::traits::IpBan;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone, Debug)]
pub struct GameController {
    cfg: Arc<GSServerConfig>,
    pub exp_table: ExpTable,
    pub action_list: ActionList,
    pub skill_trees_data: SkillTreesData,
    pub class_templates: Arc<ClassTemplates>,
    ls_actor: Arc<RwLock<Option<ActorRef<LoginServerClient>>>>,
    online_chars: DashMap<String, Option<ActorRef<PlayerClient>>>,
    pub base_stats_table: BaseStat,
    pub hero_list: DashMap<i32, character::Model>,
    pub clan_ally_manager: Arc<RwLock<ClanAllyManager>>,
}

impl GameController {
    pub async fn new(cfg: Arc<GSServerConfig>, db_pool: &DBPool) -> Self {
        let exp_table = ExpTable::load();
        let skill_trees_data = SkillTreesData::load();
        let action_list = ActionList::load();
        let class_templates = ClassTemplates::load();
        let base_stats = BaseStat::load();
        GameController {
            exp_table,
            cfg,
            ls_actor: Arc::new(RwLock::new(None)),
            action_list,
            skill_trees_data,
            base_stats_table: base_stats,
            class_templates: Arc::new(class_templates),
            hero_list: DashMap::new(),
            online_chars: DashMap::new(),
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
        self.online_chars
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }
    pub fn add_online_account(
        &self,
        account: &str,
        pl: Option<ActorRef<PlayerClient>>,
    ) -> Option<ActorRef<PlayerClient>> {
        self.online_chars.insert(account.to_string(), pl)?
    }
    pub fn logout_account(&self, account: &str) {
        self.online_chars.remove(account);
        info!("Logged out online account: {}", account);
    }
    pub async fn add_player_to_world(
        &self,
        p: &Player,
        actor_ref: &ActorRef<PlayerClient>,
    ) -> anyhow::Result<()> {
        let ci = CharInfo::new(p, &self.cfg)?;
        //todo: implement filtering logic to send only to visible players
        for entry in &self.online_chars {
            if let Some(pl_actor) = entry.value()
                && pl_actor.id() != actor_ref.id()
            {
                let char_info = pl_actor.ask(GetCharInfo).await?;
                //tell a player about a new player that just has spawned
                pl_actor.tell(HandleOutboundPacket { packet: ci.clone() }).await?;
                //tell the player that has just spawned about existing player
                actor_ref
                    .tell(HandleOutboundPacket { packet: char_info })
                    .await?;
            }
        }
        Ok(())
    }
    pub async fn broadcast_packet<F>(
        &self,
        packet: impl SendablePacket + Clone + Send + 'static,
        filter: F,
    ) where
        F: Fn(&String, &ActorRef<PlayerClient>) -> bool,
    {
        for entry in &self.online_chars {
            let account = entry.key();
            if let Some(pl_actor) = entry.value() {
                // Apply the filter function
                if !filter(account, pl_actor) {
                    continue;
                }
                let pkt = packet.clone();
                if let Err(e) = pl_actor.tell(HandleOutboundPacket { packet: pkt }).await {
                    warn!(
                        "Failed to broadcast packet to player, for account {account}, cause: {e}"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
impl GameController {
    pub fn from_config(cfg: Arc<GSServerConfig>) -> Self {
        let exp_table = ExpTable::load();
        let action_list = ActionList::load();
        let skill_trees_data = SkillTreesData::load();
        let class_templates = ClassTemplates::load();
        let base_stats = BaseStat::load();
        GameController {
            exp_table,
            cfg,
            action_list,
            skill_trees_data,
            ls_actor: Arc::new(RwLock::new(None)),
            base_stats_table: base_stats,
            class_templates: Arc::new(class_templates),
            hero_list: DashMap::new(),
            online_chars: DashMap::new(),
            clan_ally_manager: Arc::new(RwLock::new(ClanAllyManager::default())),
        }
    }
}
impl IpBan for GameController {
    fn is_ip_banned(&self, _: &str) -> bool {
        false
    }
}
