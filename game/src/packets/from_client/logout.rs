use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::ReadablePacket;
use tracing::{error, info, instrument};

#[derive(Debug, Clone)]
pub struct Logout;

impl ReadablePacket for Logout {
    const PACKET_ID: u8 = 0x00;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<Logout> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, ctx))]
    async fn handle(
        &mut self,
        msg: Logout,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo handle proper logout mechanism: olympiad,
        // in battle state, on RB and so on, offline trade, etc...

        // evaluate user before using macro
        let user = self.try_get_user()?;
        // todo: there is a bug when ? doesn't propagate error inside macro
        info!("Player logged out: {user:}");
        let err = ctx.actor_ref().stop_gracefully().await;
        if let Err(e) = err {
            error!("Error while stopping actor: {e:?}");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::GameController;
    use crate::test_utils::test::{spawn_custom_player_client_actor, spawn_player_client_actor};
    use entities::test_factories::factories::user_factory;
    use l2_core::config::gs::GSServerConfig;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[tokio::test]
    async fn test_err() {
        let pool = get_test_db().await;
        let packet = Logout {};
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg));
        controller.add_online_account(String::from("test"));
        let _user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        let pl_actor = spawn_player_client_actor(controller, pool, r, w).await;
        let res = pl_actor.ask(packet).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_ok() {
        let pool = get_test_db().await;
        let packet = Logout {};
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg));
        controller.add_online_account(String::from("test"));
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        let mut pl_client = PlayerClient::new(Ipv4Addr::LOCALHOST, controller, pool);
        pl_client.set_user(user);
        let pl_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = pl_actor.ask(packet).await;
        assert!(res.is_ok());
    }
}
