use crate::client_thread::{ClientHandler, ClientStatus};
use crate::packets::to_client::CharSelected;
use crate::packets::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

#[derive(Debug, Clone)]
pub struct SelectChar {
    char_slot: i32,
}

impl ReadablePacket for SelectChar {
    const PACKET_ID: u8 = 0x12;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            char_slot: buffer.read_i32()?,
        })
        //unknown part
        //buffer.read_i16()
        //buffer.read_i32()
        //buffer.read_i32()
    }
}

#[async_trait]
impl HandleablePacket for SelectChar {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        //todo flood protection, ban check, maybe set online status in DB?
        if handler.get_selected_char_slot().is_some() {
            bail!("Char already selected")
        }
        handler.try_get_session_key()?;
        handler.try_get_user()?;
        handler.try_get_char_by_slot_id(self.char_slot)?;
        let game_time = handler.get_controller().get_game_time();
        handler.set_status(ClientStatus::Entering);
        handler.select_char(self.char_slot);
        let char_info = handler.try_get_char_by_slot_id(self.char_slot)?;
        handler
            .send_packet(Box::new(CharSelected::new(
                char_info,
                handler.try_get_session_key()?.get_play_session_id(),
                game_time
            )?))
            .await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use ntest::timeout;
    use tokio::io::{split, AsyncWriteExt};
    use entities::dao::char_info::CharacterInfo;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::config::gs::GSServer;
    use l2_core::session::SessionKey;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use test_utils::utils::get_test_db;
    use crate::controller::Controller;
    use super::*;
    #[tokio::test]
    #[timeout(3000)]
    async fn test_handle() {
        let pool = get_test_db().await;
        let pack = SelectChar { char_slot: 0 };
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(Controller::from_config(cfg));
        controller.add_online_account(String::from("test"));
        let user = user_factory(&pool, |mut u| {
            u.username = String::from("test");
            u
        })
            .await;
        let char_model = char_factory(&pool, |mut u| {
            u.user_id = user.id;
            u.name = "Char".to_owned();
            u
        })
            .await;
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool.clone(), controller);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
        ch.set_session_key(SessionKey::new());
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
        ch.set_user(user);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_err());
        ch.set_account_chars(vec![CharacterInfo::new(char_model, vec![]).unwrap()]);
        let res = pack.handle(&mut ch).await;
        assert!(res.is_ok());
        client.shutdown().await.unwrap();
    }
}