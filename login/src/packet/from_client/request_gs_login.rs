use crate::login_client::LoginClient;
use crate::packet::to_client::PlayOk;
use anyhow::bail;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::session::SessionKey;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct RequestGSLogin {
    pub s_key_1: i32,
    pub s_key_2: i32,
    pub server_id: u8,
}
impl RequestGSLogin {
    pub fn check_session(&self, session_key: &SessionKey) -> anyhow::Result<()> {
        if !session_key.check_session(self.s_key_1, self.s_key_2) {
            bail!("Session key check failed");
        }
        Ok(())
    }
}
impl Message<RequestGSLogin> for LoginClient {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        msg: RequestGSLogin,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        msg.check_session(&self.session_key)?;
        let acc_name = self
            .account_name
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Account name not found"))?;
        let _ = self.controller.with_player(acc_name, |p| {
            p.gs_id = Some(msg.server_id);
            true
        });
        self.send_packet(PlayOk::new(&self.session_key)?)
            .await?;
        Ok(())
    }
}
impl ReadablePacket for RequestGSLogin {
    const PACKET_ID: u8 = 0x02;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            s_key_1: buffer.read_i32()?,
            s_key_2: buffer.read_i32()?,
            server_id: buffer.read_byte()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use crate::test_utils::test::{spawn_custom_login_client_actor, spawn_login_client_actor};
    use l2_core::config::login::LoginServerConfig;
    use l2_core::session::SessionKey;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[test]
    fn request_gs_login_read() {
        let bytes = [1, 0, 0, 0, 2, 0, 0, 0, 8];
        let packet = RequestGSLogin::read(BytesMut::from(&bytes[..])).unwrap();
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
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let (r, w) = split(server);
        let player_actor = spawn_login_client_actor(cloned_lc, db_pool, r, w).await;
        let result = player_actor.ask(packet).await;
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
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let session_key = SessionKey {
            play_ok1: 8,
            play_ok2: 9,
            login_ok1: 3,
            login_ok2: 4,
        };
        let mut login_client = LoginClient::new(ip, cloned_lc, db_pool.clone());
        login_client.account_name = Some("test".to_string());
        login_client.session_key = session_key;
        let player_actor =
            spawn_custom_login_client_actor(lc, db_pool, r, w, Some(login_client)).await;
        let result = player_actor.ask(packet).await;
        assert!(result.is_ok());
    }
}
