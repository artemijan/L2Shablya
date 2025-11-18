use crate::game_objects::player::Player;

#[derive(Debug, Clone, Copy, Default)]
pub enum PartyLoot {
    Random,
    ByTurn,
    RandomIncludingSpoil,
    ByTurnIncludingSpoil,
    #[default]
    FindersKeepers,
}

impl PartialEq for Party {
    fn eq(&self, other: &Self) -> bool {
        self.get_leader_id() == other.get_leader_id()
    }
}

impl Eq for Party {}

#[derive(Debug, Clone)]
pub struct Party {
    loot: PartyLoot,
    players: Vec<Player>,
}

impl Party {
    pub fn new(leader: Player) -> Self {
        Self {
            loot: PartyLoot::default(),
            players: vec![leader.clone()],
        }
    }

    #[must_use]
    pub fn get_leader(&self) -> &Player {
        &self
            .players
            .first()
            .unwrap_or_else(|| panic!("Programming error: Party has no leader"))
    }
    #[must_use]
    pub fn get_leader_id(&self) -> i32 {
        self.get_leader().char_model.id
    }

    #[must_use]
    pub fn get_players(&self) -> &Vec<Player> {
        &self.players
    }
    #[must_use]
    pub fn index_of(&self, player_id: i32) -> Option<u32> {
        self.players
            .iter()
            .enumerate()
            .find(|(_, p)| p.char_model.id == player_id)
            .map(|(i, _)| i)
            .map(u32::try_from)?
            .ok()
    }
}
