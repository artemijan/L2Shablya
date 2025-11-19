use crate::gs_client::GameServerClient;
use kameo::message::{Context, Message};
use l2_core::shared_packets::{gs_2_ls::PlayerAuthRequest, ls_2_gs::PlayerAuthResponse};
use l2_core::traits::ServerToServer;
use tracing::{error, instrument};

impl Message<PlayerAuthRequest> for GameServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: PlayerAuthRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let show_license = self.lc.get_config().client.show_licence;
        let operation_ok = self.lc.with_player(&msg.account_name, |pl| {
            if let Some(session) = &pl.session
                && session.equals(&msg.session, show_license)
            {
                pl.game_server = self.server_id;
                return true;
            }
            false // the operation wasn't successful
        });
        self.send_packet(PlayerAuthResponse::new(&msg.account_name, operation_ok))
            .await?;
        if !operation_ok {
            error!("Not authed, so closing connection.");
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use crate::test_utils::test::spawn_gs_client_actor;
    use l2_core::config::login::LoginServerConfig;
    use l2_core::session::SessionKey;
    use l2_core::traits::ServerConfig;
    use std::slice::from_ref;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[tokio::test]
    async fn handle_auth_user_not_found() {
        let sk = SessionKey::new();
        let packet = PlayerAuthRequest::new("admin", sk.clone()).unwrap();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let (r, w) = split(server);
        let gs_actor = spawn_gs_client_actor(lc, db_pool, r, w).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_ok()); //session should not be closed
    }
    #[tokio::test]
    async fn handle_auth_user_ok() {
        let sk = SessionKey::new();
        let packet = PlayerAuthRequest::new("admin", sk.clone()).unwrap();
        let db_pool = get_test_db().await;
        let acc = "admin".to_string();
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        lc.on_players_in_game(1, from_ref(&acc)); // hack to insert players
        lc.with_player(&acc, |pl| {
            pl.session = Some(sk.clone());
            true
        });
        let (r, w) = split(server);
        let gs_actor = spawn_gs_client_actor(lc, db_pool, r, w).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn handle_auth_user_session_not_set() {
        let sk = SessionKey::new();
        let packet = PlayerAuthRequest::new("admin", sk.clone()).unwrap();
        let db_pool = get_test_db().await;
        let acc = "admin".to_string();
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        lc.on_players_in_game(1, from_ref(&acc)); // hack to insert players
        let (r, w) = split(server);
        let gs_actor = spawn_gs_client_actor(lc, db_pool, r, w).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_ok()); // gs session should not be closed
    }
}
