use crate::gs_client::GameServerClient;
use crate::{dto::game_server::GSInfo, enums};
use anyhow::bail;
use kameo::message::{Context, Message};
use l2_core::constants::try_get_server_name_by_id;
use l2_core::shared_packets::{common::GSLoginFail, gs_2_ls::RequestAuthGS, ls_2_gs::AuthGS};
use tracing::instrument;
use l2_core::traits::ServerToServer;

impl Message<RequestAuthGS> for GameServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RequestAuthGS,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let gsi = GSInfo::new(
            msg.desired_id,
            msg.accept_alternative_id,
            msg.port,
            0,
            true,
            0,
            0,
            false,
            msg.max_players,
            msg.hex_id.clone(),
            &msg.hosts,
        )?;
        match self.lc.register_gs(gsi) {
            Ok(desired_id) => {
                self.set_connection_state(&enums::GS::Authed).await?;
                let server_name = try_get_server_name_by_id(desired_id)?;
                self.server_id = Some(desired_id);
                self.send_packet(
                    AuthGS::new(desired_id, server_name),
                )
                .await
            }
            Err(e) => {
                let err_msg = format!(
                    "Failed to register game server with id {:}, fail reason {:?}",
                    msg.desired_id, e
                );
                self.send_packet(GSLoginFail::new(e)?).await?;
                bail!(err_msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::LoginController;
    use crate::enums::GS;
    use crate::gs_client::GameServerClient;
    use crate::test_utils::test::{spawn_custom_gs_client_actor, spawn_gs_client_actor};
    use l2_core::config::login::LoginServerConfig;
    use l2_core::shared_packets::gs_2_ls::RequestAuthGS;
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
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let (r, w) = split(server);
        let actor = spawn_gs_client_actor(lc, db_pool, r, w).await;
        let res = actor.ask(packet).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn handler_auth_ok() {
        let packet = get_packet();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::LOCALHOST;
        let (r, w) = split(server);
        let mut gs_client = GameServerClient::new(ip, cloned_lc, db_pool.clone());
        gs_client
            .set_connection_state(&GS::Connected)
            .await
            .unwrap();
        gs_client
            .set_connection_state(&GS::BfConnected)
            .await
            .unwrap();
        let actor = spawn_custom_gs_client_actor(lc, db_pool, r, w, Some(gs_client)).await;
        let res = actor.ask(packet).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn handler_auth_err_register_twice() {
        let packet = get_packet();
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::LOCALHOST;
        let (r, w) = split(server);
        let mut gs_client = GameServerClient::new(ip, cloned_lc, db_pool.clone());
        gs_client
            .set_connection_state(&GS::Connected)
            .await
            .unwrap();
        gs_client
            .set_connection_state(&GS::BfConnected)
            .await
            .unwrap();
        let actor = spawn_custom_gs_client_actor(lc, db_pool, r, w, Some(gs_client)).await;
        let res = actor.ask(packet.clone()).await;
        assert!(res.is_ok());
        let res = actor.ask(packet).await;
        assert!(res.is_err());
    }
}
