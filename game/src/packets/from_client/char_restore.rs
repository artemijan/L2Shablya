use crate::packets::to_client::CharSelectionInfo;
use crate::pl_client::PlayerClient;
use anyhow::bail;
use bytes::BytesMut;
use entities::entities::character;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct RestoreChar {
    char_slot: i32,
}

impl ReadablePacket for RestoreChar {
    const PACKET_ID: u8 = 0x7B;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            char_slot: buffer.read_i32()?,
        })
    }
}
impl Message<RestoreChar> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RestoreChar,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo flood protection

        // this must be done first before we try to restore char,
        // because, if it's cheating, we need to handle it before we do changes
        self.try_get_session_key()?;
        self.try_get_account_chars()?;
        self.try_get_user()?;
        let pool = self.db_pool.clone();
        self.with_char_by_slot_id(msg.char_slot, |model| async move {
            if model.delete_at.is_none() {
                bail!("Possible cheat: The char can't be restored as it is not deleted");
            }
            let new_char = character::Model::restore_char(&pool, model).await?;
            Ok(new_char)
        })
        .await?;
        let sk = self.try_get_session_key()?;
        let chars = self.try_get_account_chars()?;
        let user_name = &self.try_get_user()?.username;
        let p =
            CharSelectionInfo::new(user_name, sk.get_play_session_id(), &self.controller, chars)?;
        self.send_packet(p.buffer).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::GameController;
    use crate::pl_client::ClientStatus;
    use crate::test_utils::test::spawn_custom_player_client_actor;
    use entities::entities::user;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServerConfig;
    use l2_core::game_objects::player::Player;
    use l2_core::session::SessionKey;
    use l2_core::traits::ServerConfig;
    use sea_orm::sqlx::types::chrono::Utc;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::{get_test_db, DBPool};
    use tokio::io::split;

    async fn prepare() -> PlayerClient {
        let pool = get_test_db().await;
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg));
        controller.add_online_account(String::from("test"));
        PlayerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone())
    }
    async fn user(pool: &DBPool) -> user::Model {
        user_factory(pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await
    }
    async fn char(pool: &DBPool, user_id: i32) -> character::Model {
        char_factory(pool, |mut u| {
            u.user_id = user_id;
            u.name = "TestChar".to_owned();
            u.delete_at = Some(Utc::now().into());
            u
        })
        .await
    }
    #[tokio::test]
    async fn test_and_handle_error() {
        let pack = RestoreChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut player_client = prepare().await;
        player_client
            .controller
            .add_online_account(String::from("test"));
        let user = user(&player_client.db_pool).await;
        player_client.set_status(ClientStatus::Authenticated);
        player_client.set_user(user);
        let player_actor = spawn_custom_player_client_actor(
            player_client.controller.clone(),
            player_client.db_pool.clone(),
            r,
            w,
            Some(player_client),
        )
        .await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn test_and_handle_error_when_empty_char_list() {
        let pack = RestoreChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare().await;
        pl_client
            .controller
            .add_online_account(String::from("test"));
        let user = user(&pl_client.db_pool).await;
        pl_client.set_status(ClientStatus::Authenticated);
        pl_client.set_user(user);
        pl_client.set_account_chars(vec![]);
        let player_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = player_actor.ask(pack).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_and_handle_error_char_is_not_deleted() {
        let pack = RestoreChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare().await;
        pl_client
            .controller
            .add_online_account(String::from("test"));
        pl_client.set_status(ClientStatus::Authenticated);
        let user = user(&pl_client.db_pool).await;
        let char_model = char(&pl_client.db_pool, user.id).await;
        pl_client.set_user(user);
        pl_client.set_account_chars(vec![Player::new(char_model, vec![])]);
        let pl_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = pl_actor.ask(pack).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn test_and_handle_no_session_key_error() {
        let pack = RestoreChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare().await;
        pl_client
            .controller
            .add_online_account(String::from("test"));
        pl_client.set_status(ClientStatus::Authenticated);
        let user = user(&pl_client.db_pool).await;
        let char_model = char(&pl_client.db_pool, user.id).await;
        pl_client.set_user(user);
        pl_client.set_account_chars(vec![Player::new(char_model, vec![])]);
        let pl_actor = spawn_custom_player_client_actor(
            pl_client.controller.clone(),
            pl_client.db_pool.clone(),
            r,
            w,
            Some(pl_client),
        )
        .await;
        let res = pl_actor.ask(pack).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn test_and_handle_ok() {
        let pack = RestoreChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare().await;
        pl_client.set_status(ClientStatus::Authenticated);
        let user = user(&pl_client.db_pool).await;
        let char_model = char(&pl_client.db_pool, user.id).await;
        pl_client.set_user(user);
        pl_client.set_account_chars(vec![Player::new(char_model, vec![])]);
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
        assert!(res.is_ok());
    }
}
