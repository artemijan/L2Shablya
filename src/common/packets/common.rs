
use crate::common::packets::write::SendablePacketBuffer;
use async_trait::async_trait;
use crate::common::packets::error::PacketRun;
use num_enum::TryFromPrimitive;
use std::{fmt::Debug, net::Ipv4Addr};

use super::gs_2_ls::ReplyChars;

pub type PacketResult = Result<Option<Box<dyn SendablePacket>>, PacketRun>;

pub trait SendablePacket: Debug + Send + Sync {
    fn get_bytes_mut(&mut self) -> &mut [u8] {
        let buff = self.get_buffer_mut();
        buff.get_data_mut()
    }
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer;
}

pub trait ReadablePacket: Debug + Send + Sync {
    fn read(data: &[u8]) -> Option<Self>
    where
        Self: Sized + ReadablePacket;
}
#[repr(u8)]
#[allow(unused)]
#[derive(Clone, Debug)]
pub enum LoginServerOpcodes {
    Init = 0x00,
    LoginOk = 0x03,
    ServerList = 0x04,
    GgAuth = 0x0b,
    LoginFail = 0x01,
    AccountKicked = 0x02,
    PlayFail = 0x06,
    PlayOk = 0x07,
    PiAgreementCheck = 0x11,
    PiAgreementAck = 0x12,
    LoginOptFail = 0x0D,
}


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
pub trait HandlablePacket: Debug + Send {
    type HandlerType;
    async fn handle(
        &self,
        ch: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun>;
}

#[derive(Debug)]
pub enum PacketType {
    ReplyChars(ReplyChars),
}
