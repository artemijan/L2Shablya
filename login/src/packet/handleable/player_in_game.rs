use crate::gs_client::GameServerClient;
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::PlayerInGame;
use tracing::instrument;

impl Message<PlayerInGame> for GameServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: PlayerInGame,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.lc
            .on_players_in_game(self.try_get_server_id()?, &msg.accounts);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use crate::test_utils::test::{spawn_custom_gs_client_actor, spawn_gs_client_actor};
    use l2_core::config::login::LoginServerConfig;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[tokio::test]
    async fn handle_gs_not_set() {
        let packet = PlayerInGame::new(&["admin".to_string(), "test".to_string()]).unwrap();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let (r, w) = split(server);
        let gs_actor = spawn_gs_client_actor(lc, db_pool, r, w).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn handle_ok() {
        let packet = PlayerInGame::new(&["admin".to_string(), "test".to_string()]).unwrap();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut gs_client = GameServerClient::new(ip, lc.clone(), db_pool.clone());
        gs_client.server_id = Some(1);
        let gs_actor =
            spawn_custom_gs_client_actor(lc.clone(), db_pool, r, w, Some(gs_client)).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_ok());
        assert!(lc.get_player("admin").is_some());
        assert!(lc.get_player("test").is_some());
    }
}
