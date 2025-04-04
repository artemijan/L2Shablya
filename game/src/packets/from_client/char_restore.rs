use crate::client_thread::ClientHandler;
use crate::packets::to_client::CharSelectionInfo;
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use entities::entities::character;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

#[derive(Debug, Clone)]
pub struct RestoreChar {
    char_slot: i32,
}

impl ReadablePacket for RestoreChar {
    const PACKET_ID: u8 = 0x7B;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            char_slot: buffer.read_i32()?,
        })
    }
}

#[async_trait]
impl HandleablePacket for RestoreChar {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        //todo flood protection

        let db_pool = handler.get_db_pool().clone();
        // this must be done first before we try to restore char,
        // because if it's cheating we need to handle it before we do changes
        handler.try_get_session_key()?;
        handler.try_get_account_chars()?;
        handler.try_get_user()?;
        handler
            .with_char_by_slot_id(self.char_slot, |model| async move {
                if model.delete_at.is_none() {
                    bail!("Possible cheat: The char can't be restored as it is not deleted");
                }
                let new_char = character::Model::restore_char(&db_pool, model).await?;
                Ok(new_char)
            })
            .await?;
        let controller = handler.get_controller();
        let sk = handler.try_get_session_key()?;
        let chars = handler.try_get_account_chars()?;
        let user_name = &handler.try_get_user()?.username;
        let p = CharSelectionInfo::new(user_name, sk.get_play_session_id(), controller, chars)?;
        handler.send_packet(Box::new(p)).await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::client_thread::ClientStatus;
    use crate::controller::Controller;
    use entities::dao::char_info::CharacterInfo;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServer;
    use l2_core::session::SessionKey;
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use sea_orm::sqlx::types::chrono::Utc;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncWriteExt};
    use l2_core::traits::handlers::PacketHandler;

    #[tokio::test]
    #[timeout(3000)]
    async fn test_and_handle_error() {
        let pool = get_test_db().await;
        let pack = RestoreChar { char_slot: 0 };
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(Controller::from_config(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        ch.set_status(ClientStatus::Authenticated);
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        ch.set_user(user);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
        ch.set_account_chars(vec![]);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
        client.shutdown().await.unwrap();
    }
    #[tokio::test]
    #[timeout(3000)]
    async fn test_and_handle_error_char_is_not_deleted() {
        let pool = get_test_db().await;
        let pack = RestoreChar { char_slot: 0 };
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(Controller::from_config(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        ch.set_status(ClientStatus::Authenticated);
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        let char_model = char_factory(&pool, |mut u| {
            u.user_id = user.id;
            u.name = "TestChar".to_owned();
            u
        })
        .await;
        ch.set_user(user);
        ch.set_account_chars(vec![CharacterInfo::new(char_model, vec![]).unwrap()]);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
        client.shutdown().await.unwrap();
    }
    #[tokio::test]
    #[timeout(3000)]
    async fn test_and_handle_ok() {
        let pool = get_test_db().await;
        let pack = RestoreChar { char_slot: 0 };
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(Controller::from_config(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        ch.set_status(ClientStatus::Authenticated);
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
        .await;
        let char_model = char_factory(&pool, |mut u| {
            u.user_id = user.id;
            u.name = "TestChar".to_owned();
            u.delete_at = Some(Utc::now().into());
            u
        })
        .await;
        ch.set_user(user);
        ch.set_account_chars(vec![CharacterInfo::new(char_model, vec![]).unwrap()]);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
        ch.set_session_key(SessionKey::new());
        let res = pack.handle(&mut ch).await;
        assert!(res.is_ok());
        client.shutdown().await.unwrap();
    }
}
