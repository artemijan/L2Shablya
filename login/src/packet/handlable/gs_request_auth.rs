use crate::packet::HandleablePacket;
use crate::{
    dto::game_server::GSInfo,
    gs_thread::{enums, GSHandler},
};
use anyhow::bail;
use async_trait::async_trait;
use l2_core::constants::try_get_server_name_by_id;
use l2_core::traits::handlers::PacketSender;
use l2_core::{
    shared_packets::{common::GSLoginFail, gs_2_ls::RequestAuthGS, ls_2_gs::AuthGS},
    traits::handlers::PacketHandler,
};

#[async_trait]
impl HandleablePacket for RequestAuthGS {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let gsi = GSInfo::new(
            self.desired_id,
            self.accept_alternative_id,
            self.host_reserved,
            self.port,
            true,
            0,
            true,
            0,
            0,
            false,
            self.max_players,
            self.hex_id.clone(),
            &self.hosts,
        )?;
        match gs.get_controller().register_gs(gsi) {
            Ok(desired_id) => {
                gs.set_connection_state(&enums::GS::Authed).await?;
                let server_name = try_get_server_name_by_id(desired_id)?;
                gs.server_id = Some(desired_id);
                gs.send_packet(Box::new(AuthGS::new(desired_id, server_name)))
                    .await
            }
            Err(e) => {
                let err_msg = format!(
                    "Failed to register game server with id {:}, fail reason {:?}",
                    self.desired_id, e
                );
                gs.send_packet(Box::new(GSLoginFail::new(e)?)).await?;
                bail!(err_msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::LoginController;
    use crate::gs_thread::enums::GS;
    use crate::gs_thread::GSHandler;
    use crate::packet::HandleablePacket;
    use l2_core::config::login::LoginServer;
    use l2_core::shared_packets::gs_2_ls::RequestAuthGS;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    fn get_packet() -> RequestAuthGS {
        RequestAuthGS {
            desired_id: 5,
            port: 2106,
            max_players: 5000,
            hex_id: i128::from_str_radix("-2ad66b3f483c22be097019f55c8abdf0", 16)
                .unwrap()
                .to_be_bytes()
                .to_vec(),
            hosts: vec!["127.0.0.1/0".to_string(), "127.0.0.1".to_string()],
            ..Default::default()
        }
    }
    #[tokio::test]
    async fn handler_auth_wrong_state() {
        let packet = get_packet();
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
    async fn handler_auth_ok() {
        let packet = get_packet();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        ch.set_connection_state(&GS::Connected).await.unwrap();
        ch.set_connection_state(&GS::BfConnected).await.unwrap();
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn handler_auth_err_register_twice() {
        let packet = get_packet();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        ch.set_connection_state(&GS::Connected).await.unwrap();
        ch.set_connection_state(&GS::BfConnected).await.unwrap();
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
        let res = packet.handle(&mut ch).await;
        assert!(res.is_err());
    }
}
