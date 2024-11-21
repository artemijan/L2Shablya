use crate::common::packets::common::ReadablePacket;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::packets::error;
use crate::common::traits::handlers::PacketHandler;
use crate::common::packets::ls_2_gs::AuthGS;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct GS {
    pub desired_id: u8,
    pub accept_alternative_id: bool,
    pub host_reserved: bool,
    pub port: u16,
    pub max_players: u32,
    pub hex_id: Vec<u8>,
    pub hosts: Vec<String>,
}

#[allow(clippy::cast_sign_loss)]
impl ReadablePacket for GS {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let desired_id = buffer.read_byte();
        let accept_alternative_id = buffer.read_byte() != 0;
        let host_reserved = buffer.read_byte() != 0;
        let port = buffer.read_i16() as u16;
        let max_players = buffer.read_i32() as u32;
        let mut size = buffer.read_i32();
        let hex_id = buffer.read_bytes(size as usize);
        size = buffer.read_i32() * 2;
        let hosts = buffer.read_n_strings(size as usize);
        Some(Self {
            desired_id,
            accept_alternative_id,
            host_reserved,
            port,
            max_players,
            hex_id,
            hosts,
        })
    }
}

