use crate::login_server::client_thread::ClientHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::ClientHandle;
use crate::packet::common::{ReadablePacket, SendablePacket, ServerData, ServerStatus, ServerType};
use crate::packet::to_client::ServerList;
use async_trait::async_trait;
use std::net::Ipv4Addr;
use crate::login_server::traits::PacketHandler;
use crate::packet::{error, login_fail, PlayerLoginFailReasons};

#[derive(Clone, Debug)]
pub struct RequestServerList {
    pub login_ok_1: i32,
    pub login_ok_2: i32,
}

impl ReadablePacket for RequestServerList {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        Some(RequestServerList {
            login_ok_1: buffer.read_i32(),
            login_ok_2: buffer.read_i32(),
        })
    }
}

#[async_trait]
impl ClientHandle for RequestServerList {
    async fn handle(&self, ch: &mut ClientHandler) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        let lc = ch.get_lc();
        let servers = lc.get_server_list(ch.ip).await;
        if let Some(ref acc_name) = ch.account_name {
            let mut player_option = lc.get_player(acc_name).await;
            let mut chars_on_servers = None;
            if let Some(player) = player_option {
                chars_on_servers = Some(player.chars_on_servers)
            }
            return Ok(Some(Box::new(ServerList::new(servers, 1, chars_on_servers))));
        }
        Err(error::PacketRun {
            msg: Some(format!("Login Fail, tried user: {:?}", ch.account_name)),
            response: Some(Box::new(login_fail::PlayerLogin::new(
                PlayerLoginFailReasons::ReasonUserOrPassWrong,
            ))),
        })
    }
}
