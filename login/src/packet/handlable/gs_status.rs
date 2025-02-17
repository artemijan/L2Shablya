use crate::gs_thread::GSHandler;
use crate::packet::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::constants::try_get_server_name_by_id;
use l2_core::traits::handlers::PacketSender;
use l2_core::{
    shared_packets::{
        common::{PlayerLoginFail, PlayerLoginFailReasons},
        gs_2_ls::GSStatusUpdate,
    },
    traits::handlers::PacketHandler,
};
use std::sync::Arc;
use tracing::{info, instrument};

#[async_trait]
impl HandleablePacket for GSStatusUpdate {
    type HandlerType = GSHandler;

    #[instrument(skip_all)]
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let lc = gs.get_controller();
        let mut updated = false;
        if let Some(server_id) = gs.server_id {
            updated = lc.with_gs(server_id, |gsi| {
                gsi.set_max_players(self.max_players);
                gsi.set_age_limit(self.server_age);
                gsi.use_square_brackets(self.use_square_brackets);
                gsi.set_server_type(self.server_type);
                gsi.set_server_status(self.status as i32);
            });
            info!(
                "Game server registered: {:}({server_id})",
                try_get_server_name_by_id(server_id)?
            );
        }
        if !updated {
            gs.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonAccessFailed,
            )?))
            .await?;
            bail!("Server was not found, GS id {:?}", gs.server_id);
        }
        lc.message_broker
            .register_packet_handler(gs.try_get_server_id()?, Arc::new(gs.clone()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use crate::dto::game_server::GSInfo;
    use l2_core::config::login::LoginServer;
    use l2_core::shared_packets::common::GSStatus;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use test_utils::utils::{get_test_db, test_hex_id};
    use tokio::io::split;
    #[tokio::test]
    async fn handler_gs_not_found() {
        let packet = GSStatusUpdate {
            status: GSStatus::Auto,
            max_players: 5000,
            server_type: 1,
            ..Default::default()
        };
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
    async fn handler_gs_ok() {
        let packet = GSStatusUpdate {
            status: GSStatus::Auto,
            max_players: 5000,
            server_type: 1,
            ..Default::default()
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let server_id = 1;
        lc.register_gs(
            GSInfo::new(
                server_id,
                true,
                false,
                9106,
                true,
                1,
                false,
                1,
                0,
                false,
                5000,
                test_hex_id(),
                &["192.168.0.100/8".to_string(), "192.168.0.0".to_string()],
            )
            .unwrap(),
        )
        .unwrap();
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        ch.server_id = Some(server_id);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
        assert_eq!(lc.message_broker.packet_handlers.len(), 1);
    }
}
