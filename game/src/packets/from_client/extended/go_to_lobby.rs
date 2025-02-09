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
    use l2_core::session::SessionKey;
    use l2_core::traits::handlers::PacketHandler;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use ntest::timeout;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt};
    use crate::tests::get_gs_config;

    #[tokio::test]
    #[timeout(2000)]
    pub async fn test_handle() {
        let pool = get_test_db().await;
        let pack = GoLobby;
        let user = user_factory(&pool, |u| u).await;
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = get_gs_config();
        let controller = Arc::new(Controller::new(cfg));
        let mut ch = ClientHandler::new(r, w, Ipv4Addr::LOCALHOST, pool, controller);

        let res = pack.handle(&mut ch).await;
        assert!(matches!(res, Err(e) if e.to_string() == "User not set"));
        ch.set_user(user);
        let res2 = pack.handle(&mut ch).await;
        assert!(
            matches!(res2, Err(e) if e.to_string() == "Can not go to lobby, Session is missing")
        );
        ch.set_session_key(SessionKey::new());
        let res3 = pack.handle(&mut ch).await;
        assert!(
            matches!(res3, Err(e) if e.to_string() == "Can not go to lobby, no chars were set")
        );
        ch.set_account_chars(vec![]);
        pack.handle(&mut ch).await.unwrap();
        tokio::spawn(async move {
            ch.handle_client().await.unwrap();
        });
        let mut ok_resp = [0; 18];
        client.read_exact(&mut ok_resp).await.unwrap();
        assert_eq!(ok_resp[2], 0x09);
        assert_eq!(u32::from_le_bytes(ok_resp[3..7].try_into().unwrap()), 0); //0 chars
    }
}
