use crate::gs_client::GameServerClient;
use anyhow::bail;
use kameo::message::{Context, Message};
use l2_core::constants::try_get_server_name_by_id;
use l2_core::shared_packets::{
    common::{PlayerLoginFail, PlayerLoginFailReasons},
    gs_2_ls::GSStatusUpdate,
};
use tracing::{info, instrument};
use l2_core::traits::ServerToServer;

impl Message<GSStatusUpdate> for GameServerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, ctx))]
    async fn handle(
        &mut self,
        msg: GSStatusUpdate,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let mut updated = false;
        if let Some(server_id) = self.server_id {
            updated = self.lc.with_gs(server_id, |gsi| {
                gsi.set_max_players(msg.max_players);
                gsi.set_age_limit(msg.server_age);
                gsi.use_square_brackets(msg.use_square_brackets);
                gsi.set_server_type(msg.server_type);
                gsi.set_server_status(msg.status as i32);
            });
            info!(
                "Game server registered: {:}({server_id})",
                try_get_server_name_by_id(server_id)?
            );
        }
        if !updated {
            self.send_packet(
                PlayerLoginFail::new(PlayerLoginFailReasons::ReasonAccessFailed)?,
            )
            .await?;
            bail!("Server was not found, GS id {:?}", self.server_id);
        }
        self.lc
            .gs_actors
            .insert(self.try_get_server_id()?, ctx.actor_ref());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use crate::dto::game_server::GSInfo;
    use crate::test_utils::test::{spawn_custom_gs_client_actor, spawn_gs_client_actor};
    use l2_core::config::login::LoginServerConfig;
    use l2_core::shared_packets::common::GSStatus;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
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
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let (r, w) = split(server);
        let gs_actor = spawn_gs_client_actor(lc, db_pool, r, w).await;
        let resp = gs_actor.ask(packet).await;
        assert!(resp.is_err());
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
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
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
        let ip = Ipv4Addr::LOCALHOST;
        let (r, w) = split(server);
        let mut gs_client = GameServerClient::new(ip, lc.clone(), db_pool.clone());
        gs_client.server_id = Some(server_id);
        let gs_actor =
            spawn_custom_gs_client_actor(lc.clone(), db_pool, r, w, Some(gs_client)).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_ok());
        assert_eq!(lc.gs_actors.len(), 1);
    }
}
