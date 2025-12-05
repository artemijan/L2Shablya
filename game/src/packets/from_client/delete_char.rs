use crate::packets::to_client::{CharDeleteFail, CharSelectionInfo};
use crate::pl_client::PlayerClient;
use anyhow::bail;
use bytes::BytesMut;
use entities::entities::character;
use kameo::message::{Context, Message};
use l2_core::enums::CharDeletionFailReasons;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use sea_orm::DbErr;
use tracing::{error, instrument};

#[derive(Debug, Clone)]
pub struct DeleteChar {
    char_slot: i32,
}

impl ReadablePacket for DeleteChar {
    const PACKET_ID: u8 = 0x0D;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            char_slot: buffer.read_i32()?,
        })
    }
}
impl Message<DeleteChar> for PlayerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: DeleteChar,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo handle proper deletion checks, e.g. check in clan, war, and so on
        let pool = self.db_pool.clone();
        let deletion_result = self
            .with_char_by_slot_id(msg.char_slot, |model| async move {
                if model.delete_at.is_some() {
                    bail!("Possible cheating: Char already set to be deleted")
                }
                let new_char = character::Model::delete_char(&pool, model).await?;
                Ok(new_char)
            })
            .await;
        if let Err(err) = deletion_result {
            return if let Some(db_err) = err.downcast_ref::<DbErr>() {
                let packet = CharDeleteFail::new(CharDeletionFailReasons::Unknown)?;
                error!("DB error while deleting a character {db_err:?}");
                self.send_packet(packet).await
            } else {
                Err(err)
            };
        }
        let sk = self.try_get_session_key()?;
        let chars = self.try_get_account_chars()?;
        let user_name = &self.try_get_user()?.username;
        let p =
            CharSelectionInfo::new(user_name, sk.get_play_session_id(), &self.controller, chars)?;
        self.send_packet( p).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::GameController;
    use crate::pl_client::ClientStatus;
    use crate::test_utils::test::spawn_custom_player_client_actor;
    use entities::entities::{character, user};
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

    async fn prepare_pl() -> PlayerClient {
        let pool = get_test_db().await;
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg).await);
        PlayerClient::new(Ipv4Addr::LOCALHOST, controller.clone(), pool.clone())
    }

    async fn user(pool: &DBPool) -> user::Model {
        user_factory(pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await
    }

    // character ready to be deleted (not yet marked for deletion)
    async fn char_ready(pool: &DBPool, user_id: i32) -> character::Model {
        char_factory(pool, |mut u| {
            u.user_id = user_id;
            u.name = "TestChar".to_owned();
            u.delete_at = None;
            u
        })
        .await
    }

    // character already marked to be deleted (cheating case)
    async fn char_marked(pool: &DBPool, user_id: i32) -> character::Model {
        char_factory(pool, |mut u| {
            u.user_id = user_id;
            u.name = "TestChar".to_owned();
            u.delete_at = Some(Utc::now().into());
            u
        })
        .await
    }

    #[tokio::test]
    async fn test_handle_error_when_empty_char_list() {
        let pack = DeleteChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare_pl().await;
        pl_client
            .controller
            .add_online_account("test", None);
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
    async fn test_handle_error_char_already_marked_for_deletion() {
        let pack = DeleteChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare_pl().await;
        pl_client
            .controller
            .add_online_account("test", None);
        pl_client.set_status(ClientStatus::Authenticated);
        let user = user(&pl_client.db_pool).await;
        let char_model = char_marked(&pl_client.db_pool, user.id).await;
        pl_client.set_user(user);
        let temp = pl_client
            .controller
            .class_templates
            .try_get_template(char_model.class_id)
            .unwrap();
        pl_client.set_account_chars(vec![Player::new(char_model, vec![], temp.clone())]);
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
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_handle_no_session_key_error() {
        let pack = DeleteChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare_pl().await;
        pl_client
            .controller
            .add_online_account("test", None);
        pl_client.set_status(ClientStatus::Authenticated);
        let user = user(&pl_client.db_pool).await;
        let char_model = char_ready(&pl_client.db_pool, user.id).await;
        pl_client.set_user(user);
        let temp = pl_client
            .controller
            .class_templates
            .try_get_template(char_model.class_id)
            .unwrap();
        pl_client.set_account_chars(vec![Player::new(char_model, vec![], temp.clone())]);
        // intentionally do NOT set session key
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
    async fn test_handle_ok() {
        let pack = DeleteChar { char_slot: 0 };
        let (_client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let mut pl_client = prepare_pl().await;
        pl_client.set_status(ClientStatus::Authenticated);
        let user = user(&pl_client.db_pool).await;
        let char_model = char_ready(&pl_client.db_pool, user.id).await;
        pl_client.set_user(user);
        let temp = pl_client
            .controller
            .class_templates
            .try_get_template(char_model.class_id)
            .unwrap();
        pl_client.set_account_chars(vec![Player::new(char_model, vec![], temp.clone())]);
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
