use async_trait::async_trait;

use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use l2_core::{shared_packets::gs_2_ls::PlayerLogout, traits::handlers::PacketHandler};

#[async_trait]
impl HandleablePacket for PlayerLogout {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let lc = gs.get_controller();
        lc.on_player_logout(&self.acc);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use l2_core::config::login::LoginServer;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[tokio::test]
    async fn handle_gs_not_set() {
        let packet = PlayerLogout::new("acc").unwrap();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn handle_logout_ok() {
        let acc = String::from("admin");
        let packet = PlayerLogout::new(&acc).unwrap();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        lc.on_players_in_game(1, &[acc.clone()]); // hack to insert players
        assert!(lc.get_player("admin").is_some());
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
        assert!(lc.get_player("admin").is_none());
    }
}
