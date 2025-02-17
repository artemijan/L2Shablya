use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::traits::handlers::PacketSender;
use l2_core::{
    shared_packets::{gs_2_ls::PlayerAuthRequest, ls_2_gs::PlayerAuthResponse},
    traits::handlers::PacketHandler,
};

#[async_trait]
impl HandleablePacket for PlayerAuthRequest {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let lc = gs.get_controller();
        let show_license = lc.get_config().client.show_licence;
        let operation_ok = lc.with_player(&self.account_name, |pl| {
            if let Some(session) = &pl.session {
                if session.equals(&self.session, show_license) {
                    pl.game_server = gs.server_id;
                    return true;
                }
            }
            false // operation wasn't successful
        });
        gs.send_packet(Box::new(PlayerAuthResponse::new(
            &self.account_name,
            operation_ok,
        )))
        .await?;
        if !operation_ok {
            bail!("Not authed, so closing connection.");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use l2_core::config::login::LoginServer;
    use l2_core::session::SessionKey;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[tokio::test]
    async fn handle_auth_user_not_found() {
        let sk = SessionKey::new();
        let packet = PlayerAuthRequest::new("admin", sk.clone()).unwrap();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn handle_auth_user_ok() {
        let sk = SessionKey::new();
        let packet = PlayerAuthRequest::new("admin", sk.clone()).unwrap();
        let db_pool = get_test_db().await;
        let acc = "admin".to_string();
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        lc.on_players_in_game(1, &[acc.clone()]); // hack to insert players
        lc.with_player(&acc, |pl| {
            pl.session = Some(sk.clone());
            true
        });
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn handle_auth_user_session_not_set() {
        let sk = SessionKey::new();
        let packet = PlayerAuthRequest::new("admin", sk.clone()).unwrap();
        let db_pool = get_test_db().await;
        let acc = "admin".to_string();
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        lc.on_players_in_game(1, &[acc.clone()]); // hack to insert players
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_err());
    }
}
