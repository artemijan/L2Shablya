use crate::client_thread::ClientHandler;
use crate::packet::to_client::AuthGG;
use crate::packet::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

#[derive(Clone, Debug)]
pub struct RequestAuthGG {
    pub session_id: i32,
}

impl ReadablePacket for RequestAuthGG {
    const PACKET_ID: u8 = 0x07;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        if buffer.get_remaining_length() > 20 {
            let session_id = buffer.read_i32()?;
            // todo I don't know what is th meaning of the data below
            // let data1 = buffer.read_i32()?;
            // let data2 = buffer.read_i32()?;
            // let data3 = buffer.read_i32()?;
            // let data4 = buffer.read_i32()?;
            return Ok(Self { session_id });
        }
        bail!("Not enough data to read packet");
    }
}

#[async_trait]
impl HandleablePacket for RequestAuthGG {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> anyhow::Result<()> {
        if self.session_id != ch.get_session_id() {
            ch.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonAccessFailed,
            )?))
            .await?;
            bail!(format!("Wrong session id {}", self.session_id));
        }
        ch.send_packet(Box::new(AuthGG::new(ch.get_session_id())))
            .await?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use l2_core::config::login::LoginServer;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[test]
    fn test_read() {
        let bytes = [
            1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0,
        ];
        let packet = RequestAuthGG::read(&bytes).expect("Read failed");
        assert_eq!(packet.session_id, 1);
    }
    #[tokio::test]
    async fn test_handle_ok() {
        let mut packet = RequestAuthGG { session_id: 1 };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = ClientHandler::new(r, w, ip, db_pool, cloned_lc);
        packet.session_id = ch.get_session_id();
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn test_handle_err() {
        let packet = RequestAuthGG { session_id: 1 };
        let db_pool = get_test_db().await;
        let (_, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = ClientHandler::new(r, w, ip, db_pool, cloned_lc);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_err());
    }
}
