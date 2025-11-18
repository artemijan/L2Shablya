use dashmap::DashMap;
use entities::entities::clan_ally;
use entities::DBPool;

#[derive(Default, Clone, Debug)]
pub struct ClanAllyManager {
    db_pool: DBPool,
    pub clan_list: DashMap<i32, clan_ally::Model>,
}
impl ClanAllyManager {
    /**
    # Panics
    - If the database connection fails.

    It ia oky to panic here as we start the manager during the boot process of the application.
    */
    pub async fn new(db_pool: DBPool) -> Self {
        let clan_list = clan_ally::Model::load_all(&db_pool)
            .await
            .expect("Failed to load clan list, can not continue...")
            .into_iter()
            .map(|c| (c.id, c))
            .collect();
        Self { db_pool, clan_list }
    }
    #[must_use]
    pub fn is_clan_leader(&self, clan_id: i32, leader_id: i32) -> bool {
        self.clan_list
            .get(&clan_id)
            .is_some_and(|c| c.leader_id == leader_id)
    }
}
