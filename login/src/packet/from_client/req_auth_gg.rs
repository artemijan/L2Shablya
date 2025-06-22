use crate::login_client::LoginClient;
use crate::packet::to_client::AuthGG;
use anyhow::bail;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::shared_packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
pub struct RequestAuthGG {
    pub session_id: i32,
}

impl Message<RequestAuthGG> for LoginClient {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        msg: RequestAuthGG,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if msg.session_id != self.session_id {
            let packet = PlayerLoginFail::new(PlayerLoginFailReasons::ReasonAccessFailed)?;
            self.send_packet(packet.buffer).await?;
            bail!(format!("Wrong session id {}", msg.session_id));
        }
        let packet = AuthGG::new(self.session_id);
        self.send_packet(packet.buffer).await?;
        Ok(())
    }
}

impl ReadablePacket for RequestAuthGG {
    const PACKET_ID: u8 = 0x07;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
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

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use super::*;
    use crate::controller::LoginController;
    use crate::test_utils::test::{GetState, spawn_login_client_actor, spawn_custom_login_client_actor};
    use l2_core::config::login::LoginServerConfig;
    use l2_core::traits::ServerConfig;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;

    #[test]
    fn test_read() {
        let bytes = [
            1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0,
        ];
        let packet = RequestAuthGG::read(BytesMut::from(&bytes[..])).expect("Read failed");
        assert_eq!(packet.session_id, 1);
    }
    #[tokio::test]
    async fn test_handle_ok() {
        let mut packet = RequestAuthGG { session_id: 1 };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let (r, w) = split(server);
        let client= LoginClient::new(Ipv4Addr::LOCALHOST, lc.clone(), db_pool.clone());
        packet.session_id = client.session_id;
        let player_actor = spawn_custom_login_client_actor(cloned_lc, db_pool, r, w, Some(client)).await;
        let result = player_actor.ask(packet).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_handle_err() {
        let session_id = 1;
        let packet = RequestAuthGG { session_id };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let (r, w) = split(server);
        let player_actor = spawn_login_client_actor(cloned_lc, db_pool, r, w).await;
        let result = player_actor.ask(packet).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        let expected = format!("Wrong session id {session_id}");
        assert_eq!(expected, err);
    }
}
