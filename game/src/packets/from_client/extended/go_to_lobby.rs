use crate::client_thread::ClientHandler;
use crate::packets::to_client::CharSelectionInfo;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[derive(Debug, Clone)]
pub struct GoLobby;

impl ReadablePacket for GoLobby {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0x33);

    fn read(_: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl HandleablePacket for GoLobby {
    type HandlerType = ClientHandler;
    async fn handle(&self, handler: &mut Self::HandlerType) -> anyhow::Result<()> {
        let user_name = &handler.try_get_user()?.username;
        let sk = handler
            .get_session_key()
            .ok_or(anyhow::anyhow!("Can not go to lobby, Session is missing"))?;
        let chars = handler
            .get_account_chars()
            .ok_or(anyhow::anyhow!("Can not go to lobby, no chars were set"))?;
        let controller = handler.get_controller();
        let p = CharSelectionInfo::new(user_name, sk.get_play_session_id(), controller, chars)?;
        handler.send_packet(Box::new(p)).await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::client_thread::ClientHandler;
    use crate::controller::Controller;
    use crate::packets::from_client::extended::GoLobby;
    use crate::packets::HandleablePacket;
    use entities::test_factories::factories::user_factory;
    use l2_core::config::gs::GSServer;
    use l2_core::session::SessionKey;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt};

    #[tokio::test]
    pub async fn test_handle() {
        let pool = get_test_db().await;
        let pack = GoLobby;
        let user = user_factory(&pool, |u| u).await;
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../test_data/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool, controller);
        ch.set_user(user);
        ch.set_session_key(SessionKey::new());
        ch.set_account_chars(vec![]);
        pack.handle(&mut ch).await.unwrap();
        tokio::spawn(async move {
            ch.handle_client().await.unwrap();
        });
        let mut resp = [0; 18];
        client.read_exact(&mut resp).await.unwrap();
        println!("{resp:?}");
        assert_eq!(resp[2], 0x09);
        assert_eq!(u32::from_le_bytes(resp[3..7].try_into().unwrap()), 0); //0 chars
    }
}
