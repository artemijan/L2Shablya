use crate::client_thread::ClientHandler;
use crate::packet::to_client::ServerList;
use crate::packet::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct RequestServerList {
    pub login_ok_1: i32,
    pub login_ok_2: i32,
}

impl ReadablePacket for RequestServerList {
    const PACKET_ID: u8 = 0x05;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            login_ok_1: buffer.read_i32()?,
            login_ok_2: buffer.read_i32()?,
        })
    }
}

#[async_trait]
impl HandleablePacket for RequestServerList {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> anyhow::Result<()> {
        if let Some(ref acc_name) = ch.account_name {
            ch.send_packet(Box::new(ServerList::new(ch, acc_name)))
                .await?;
            Ok(())
        } else {
            ch.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonUserOrPassWrong,
            )?))
            .await?;
            bail!(format!("Login Fail, tried user: {:?}", ch.account_name));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use l2_core::config::login::LoginServer;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;
    use l2_core::traits::handlers::PacketHandler;

    #[test]
    fn test_read() {
        let bytes = [1, 0, 0, 0, 2, 0, 0, 0];
        let packet = RequestServerList::read(&bytes).unwrap();
        assert_eq!(packet.login_ok_1, 1);
        assert_eq!(packet.login_ok_2, 2);
    }
    #[tokio::test]
    async fn test_handle() {
        let packet = RequestServerList {
            login_ok_1: 1,
            login_ok_2: 2,
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let mut cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        cfg.client.auto_create_accounts = false;
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = ClientHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        ch.account_name = Some(String::from("admin"));
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn test_handle_err() {
        let packet = RequestServerList {
            login_ok_1: 1,
            login_ok_2: 2,
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let mut cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        cfg.client.auto_create_accounts = false;
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = ClientHandler::new(r, w, ip, db_pool.clone(), cloned_lc);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_err());
    }
}
