#[derive(Debug, Clone)]
pub struct Party {
    //todo: implement me
    players: Vec<i32>,
    leader: i32,
}

impl Party {
    #[must_use]
    pub fn get_leader(&self) -> i32 {
        self.leader
    }
    /// This method doesn't return party leader
    #[must_use]
    pub fn get_players(&self) -> &[i32] {
        &self.players
    }
}