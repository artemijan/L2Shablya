use crate::game_objects::player::Player;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, Default)]
pub enum PartyLoot {
    Random,
    ByTurn,
    RandomIncludingSpoil,
    ByTurnIncludingSpoil,
    #[default]
    FindersKeepers,
}
#[derive(Debug, Clone)]
struct PartyInner {
    loot: PartyLoot,
    players: Vec<Arc<RwLock<Player>>>,
    leader: Arc<RwLock<Player>>,
}

#[derive(Debug, Clone)]
pub struct Party(Arc<RwLock<PartyInner>>);

impl Party {
    pub fn new(leader: &Arc<RwLock<Player>>) -> Self {
        let inner = PartyInner {
            loot: PartyLoot::default(),
            players: vec![leader.clone()],
            leader: leader.clone(),
        };
        Self(Arc::new(RwLock::new(inner)))
    }
    #[must_use]
    pub async fn add_player(&self, player: Arc<RwLock<Player>>) {
        let mut inner = self.0.write().await;
        inner.players.push(player);
    }
    #[must_use]
    pub async fn get_leader(&self) -> Arc<RwLock<Player>> {
        let inner = self.0.read().await;
        inner.leader.clone()
    }
    #[must_use]
    pub async fn get_leader_id(&self) -> i32 {
        let inner = self.0.read().await;
        inner.leader.read().await.char_model.id
    }

    #[must_use]
    pub async fn get_players(&self) -> Vec<Arc<RwLock<Player>>> {
        let inner = self.0.read().await;
        inner.players.clone()
    }
}
