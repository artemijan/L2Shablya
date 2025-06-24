use crate::gs_client::GameServerClient;
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::PlayerLogout;
use tracing::{info, instrument};

impl Message<PlayerLogout> for GameServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: PlayerLogout,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.lc.on_player_logout(&msg.acc);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use crate::test_utils::test::spawn_gs_client_actor;
    use l2_core::config::login::LoginServerConfig;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[tokio::test]
    async fn handle_gs_not_set() {
        let packet = PlayerLogout::new("acc").unwrap();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let (r, w) = split(server);
        let gs_actor = spawn_gs_client_actor(lc, db_pool, r, w).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn handle_logout_ok() {
        let acc = String::from("admin");
        let packet = PlayerLogout::new(&acc).unwrap();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        lc.on_players_in_game(1, &[acc.clone()]); // hack to insert players
        assert!(lc.get_player("admin").is_some());
        let (r, w) = split(server);
        let gs_actor = spawn_gs_client_actor(lc.clone(), db_pool, r, w).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_ok());
        assert!(lc.get_player("admin").is_none());
    }
}
