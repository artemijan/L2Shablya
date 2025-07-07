use crate::login_client::LoginClient;
use crate::packet::to_client::ServerList;
use anyhow::bail;
use bytes::BytesMut;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::shared_packets::read::ReadablePacketBuffer;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct RequestServerList {
    pub login_ok_1: i32,
    pub login_ok_2: i32,
}
impl Message<RequestServerList> for LoginClient {
    type Reply = anyhow::Result<()>;

    async fn handle(
        &mut self,
        _msg: RequestServerList,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let acc_name = self.account_name.clone();
        if let Some(ref acc_name) = acc_name {
            let packet = ServerList::new(self, acc_name);
            self.send_packet(packet).await?;
            Ok(())
        } else {
            self.send_packet(
                PlayerLoginFail::new(PlayerLoginFailReasons::ReasonUserOrPassWrong)?,
            )
            .await?;
            bail!(format!("Login Fail, tried user: {:?}", self.account_name));
        }
    }
}
impl ReadablePacket for RequestServerList {
    const PACKET_ID: u8 = 0x05;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            login_ok_1: buffer.read_i32()?,
            login_ok_2: buffer.read_i32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::LoginController;
    use crate::dto::game_server::GSInfo;
    use crate::test_utils::test::{
        GetState, spawn_custom_login_client_actor, spawn_login_client_actor,
    };
    use l2_core::config::login::LoginServerConfig;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::{get_test_db, test_hex_id};
    use tokio::io::split;

    #[test]
    fn test_read() {
        let bytes = [1, 0, 0, 0, 2, 0, 0, 0];
        let packet = RequestServerList::read(BytesMut::from(&bytes[..])).unwrap();
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
        let mut cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        cfg.client.auto_create_accounts = false;
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        lc.register_gs(
            GSInfo::new(
                1,
                true,
                false,
                9106,
                true,
                1,
                false,
                1,
                0,
                false,
                5000,
                test_hex_id(),
                &["192.168.0.100/8".to_string(), "192.168.0.0".to_string()],
            )
            .unwrap(),
        )
        .unwrap();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut login_client = LoginClient::new(ip, lc.clone(), db_pool.clone());
        login_client.account_name = Some("admin".to_string());
        let player_actor =
            spawn_custom_login_client_actor(lc.clone(), db_pool.clone(), r, w, Some(login_client))
                .await;
        let result = player_actor.ask(packet).await;
        let state = player_actor.ask(GetState).await.unwrap();
        assert!(result.is_ok());
        assert_eq!(state.account_name, Some("admin".to_string()));
    }
    #[tokio::test]
    async fn test_handle_err() {
        let packet = RequestServerList {
            login_ok_1: 1,
            login_ok_2: 2,
        };
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let mut cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        cfg.client.auto_create_accounts = false;
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let player_actor = spawn_login_client_actor(cloned_lc, db_pool, r, w).await;
        let result = player_actor.ask(packet).await;
        assert!(result.is_err());
    }
}
