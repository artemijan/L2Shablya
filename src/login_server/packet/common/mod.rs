use crate::login_server::client_thread::ClientHandler;
use crate::login_server::gs_thread::GSHandler;
use crate::common::packets::error::PacketRun;
use crate::login_server::packet::from_gs::ReplyChars;
use async_trait::async_trait;
use num_enum::TryFromPrimitive;
use std::fmt::Debug;
use std::net::Ipv4Addr;
use crate::common::packets::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum ServerStatus {
    Auto = 0x00,
    Good = 0x01,
    Normal = 0x02,
    Full = 0x03,
    Down = 0x04,
    GmOnly = 0x05,
}

#[derive(Debug, Clone)]
pub struct ServerData {
    pub ip: Ipv4Addr,
    pub port: i32,
    pub age_limit: i32,
    pub pvp: bool,
    pub current_players: i32,
    pub max_players: i32,
    pub brackets: bool,
    pub clock: bool,
    pub status: Option<ServerStatus>,
    pub server_id: i32,
    pub server_type: Option<ServerType>,
}

impl ServerData {
    pub fn get_ip_octets(&self) -> [u8; 4] {
        self.ip.octets()
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(i32)]
pub enum ServerType {
    Normal = 0x01,
    Relax = 0x02,
    Test = 0x04,
    Nolabel = 0x08,
    CreationRestricted = 0x10,
    Event = 0x20,
    Free = 0x40,
}

#[async_trait]
pub trait ClientHandle: Debug + Send {
    async fn handle(
        &self,
        ch: &mut ClientHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun>;
}

#[async_trait]
pub trait GSHandle: Debug + Send {
    async fn handle(
        &self,
        gs: &mut GSHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun>;
}

#[derive(Debug)]
pub enum PacketType {
    ReplyChars(ReplyChars),
}
