use crate::client_thread::ClientHandler;
use crate::packet::to_client::PlayOk;
use crate::packet::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct RequestGSLogin {
    pub s_key_1: i32,
    pub s_key_2: i32,
    pub server_id: u8,
}

impl ReadablePacket for RequestGSLogin {
    const PACKET_ID: u8 = 0x02;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            s_key_1: buffer.read_i32()?,
            s_key_2: buffer.read_i32()?,
            server_id: buffer.read_byte()?,
        })
    }
}

#[async_trait]
impl HandleablePacket for RequestGSLogin {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> anyhow::Result<()> {
        ch.check_session(self.s_key_1, self.s_key_2)?;
        ch.send_packet(Box::new(PlayOk::new(ch.get_session_key())?))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use l2_core::config::login::LoginServer;
    use l2_core::session::SessionKey;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;
    #[test]
    fn request_gs_login_read() {
        let bytes = [1, 0, 0, 0, 2, 0, 0, 0, 8];
        let packet = RequestGSLogin::read(&bytes).unwrap();
        assert_eq!(packet.s_key_1, 1);
        assert_eq!(packet.s_key_2, 2);
        assert_eq!(packet.server_id, 8);
    }
    #[tokio::test]
    async fn request_gs_login_handle_err() {
        let packet = RequestGSLogin {
            s_key_1: 3,
            s_key_2: 4,
            server_id: 1,
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = ClientHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        let result = packet.handle(&mut ch).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn request_gs_login_handle_ok() {
        let packet = RequestGSLogin {
            s_key_1: 3,
            s_key_2: 4,
            server_id: 1,
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = ClientHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        ch.set_session_key(SessionKey {
            play_ok1: 8,
            play_ok2: 9,
            login_ok1: 3,
            login_ok2: 4,
        });
        let result = packet.handle(&mut ch).await;
        assert!(result.is_ok());
    }
}
