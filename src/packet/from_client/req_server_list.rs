use crate::login_server::client_thread::ClientHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::ClientHandle;
use crate::packet::common::{ReadablePacket, SendablePacket, ServerData, ServerStatus, ServerType};
use crate::packet::error::PacketRun;
use crate::packet::to_client::ServerList;
use async_trait::async_trait;
use std::net::Ipv4Addr;

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
    async fn handle(&self, _: &mut ClientHandler) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        let servers = vec![ServerData {
            ip: Ipv4Addr::from([192, 168, 20, 102]),
            port: 7777,
            age_limit: 0,
            pvp: false,
            current_players: 0,
            max_players: 0,
            brackets: false,
            clock: true,
            status: ServerStatus::Good,
            server_id: 1,
            server_type: ServerType::Normal,
        }];
        Ok(Some(Box::new(ServerList::new(servers, 0, 1))))
    }
}
