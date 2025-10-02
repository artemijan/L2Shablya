use crate::packets::to_client::CharSelectionInfo;
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use tracing::instrument;
use l2_core::shared_packets::common::ReadablePacket;

#[derive(Debug, Clone)]
pub struct GoLobby;

impl ReadablePacket for GoLobby {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0x33);

    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<GoLobby> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _msg: GoLobby,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let user_name = &self.try_get_user()?.username;
        let sk = self
            .get_session_key()
            .ok_or(anyhow::anyhow!("Can not go to lobby, Session is missing"))?;
        let chars = self
            .get_account_chars()
            .ok_or(anyhow::anyhow!("Can not go to lobby, no chars were set"))?;
        let p =
            CharSelectionInfo::new(user_name, sk.get_play_session_id(), &self.controller, chars)?;
        self.send_packet(p).await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::controller::GameController;
    use crate::packets::from_client::extended::GoLobby;
    use crate::pl_client::PlayerClient;
    use crate::test_utils::test::{
        get_gs_config, spawn_custom_player_client_actor, spawn_player_client_actor,
    };
    use entities::test_factories::factories::user_factory;
    use l2_core::session::SessionKey;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use ntest::timeout;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt};

    #[tokio::test]
    pub async fn test_handle_no_user() {
        let pool = get_test_db().await;
        let pack = GoLobby;
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = get_gs_config();
        let controller = Arc::new(GameController::from_config(Arc::new(cfg)));
        let pl_actor = spawn_player_client_actor(controller, pool, r, w).await;
        let res = pl_actor.ask(pack).await;
        assert!(matches!(res, Err(e) if e.to_string() == "User not set"));
    }
    #[tokio::test]
    pub async fn test_handle_no_session() {
        let pool = get_test_db().await;
        let pack = GoLobby;
        let user = user_factory(&pool, |u| u).await;
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = get_gs_config();
        let controller = Arc::new(GameController::from_config(Arc::new(cfg)));
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
        let res = pl_actor.ask(pack).await;
        assert!(
            matches!(res, Err(e) if e.to_string() == "Can not go to lobby, Session is missing")
        );
    }
    #[tokio::test]
    pub async fn test_handle_no_chars() {
        let pool = get_test_db().await;
        let pack = GoLobby;
        let user = user_factory(&pool, |u| u).await;
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = get_gs_config();
        let controller = Arc::new(GameController::from_config(Arc::new(cfg)));
        let mut pl_client = PlayerClient::new(Ipv4Addr::LOCALHOST, controller, pool);
        pl_client.set_user(user);
        pl_client.set_session_key(SessionKey::new());
        let pl_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = pl_actor.ask(pack).await;
        assert!(matches!(res, Err(e) if e.to_string() == "Can not go to lobby, no chars were set"));
    }
    #[tokio::test]
    pub async fn test_handle_ok() {
        let pool = get_test_db().await;
        let pack = GoLobby;
        let user = user_factory(&pool, |u| u).await;
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = get_gs_config();
        let controller = Arc::new(GameController::from_config(Arc::new(cfg)));
        let mut pl_client = Play\erClient::new(Ipv4Addr::LOCALHOST, controller, pool);
        pl_client.set_user(user);
        pl_client.set_session_key(SessionKey::new());
        pl_client.set_account_chars(vec![]);
        let pl_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = pl_actor.ask(pack).await;
        assert!(res.is_ok());
        let mut ok_resp = [0; 18];
        client.read_exact(&mut ok_resp).await.unwrap();
        assert_eq!(ok_resp[2], 0x09);
        assert_eq!(u32::from_le_bytes(ok_resp[3..7].try_into().unwrap()), 0); //0 chars
    }
}
