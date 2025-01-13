use crate::config::gs::GSServer;
use crate::shared_packets::common::{ReadablePacket, SendablePacket};
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use num_traits::ToBytes;

#[derive(Clone, Debug, Default)]
pub struct RequestAuthGS {
    buffer: SendablePacketBuffer,
    pub desired_id: u8,
    pub accept_alternative_id: bool,
    pub host_reserved: bool,
    pub port: u16,
    pub max_players: u32,
    pub hex_id: Vec<u8>,
    pub hosts: Vec<String>,
}

impl RequestAuthGS {
    ///
    /// # Errors
    /// - if writing packet bytes is not possible, for example when packet size is too big
    pub fn new(cfg: &GSServer) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            desired_id: cfg.server_id,
            accept_alternative_id: cfg.accept_alternative_id,
            host_reserved: cfg.host_reserved,
            port: cfg.listeners.clients.connection.port,
            max_players: cfg.max_players,
            hex_id: cfg.hex_id.to_be_bytes(),
            hosts: cfg.get_hosts(),
        };
        inst.write_all()?;
        Ok(inst)
    }
    #[allow(clippy::cast_possible_truncation)]
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(0x01)?; // packet id
        self.buffer.write(self.desired_id)?;
        self.buffer.write_bool(self.accept_alternative_id)?;
        self.buffer.write_bool(self.host_reserved)?;
        self.buffer.write_u16(self.port)?;
        self.buffer.write_u32(self.max_players)?;
        self.buffer.write_u32(self.hex_id.len() as u32)?;
        self.buffer.write_bytes(&self.hex_id)?;
        self.buffer.write_u32((self.hosts.len() / 2) as u32)?; // we cut it by half because it's actually a pair of network/ip
        for h in &self.hosts {
            self.buffer.write_string(Some(h))?;
        }
        Ok(())
    }
}

#[allow(clippy::cast_sign_loss)]
impl ReadablePacket for RequestAuthGS {
    const PACKET_ID: u8 = 0x01;

    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer: ReadablePacketBuffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let desired_id = buffer.read_byte();
        let accept_alternative_id = buffer.read_boolean();
        let host_reserved = buffer.read_boolean();
        let port = buffer.read_u16();
        let max_players = buffer.read_u32();
        let mut size = buffer.read_u32();
        let hex_id = buffer.read_bytes(size as usize);
        size = buffer.read_u32() * 2;
        let hosts = buffer.read_n_strings(size as usize);
        Some(Self {
            buffer: SendablePacketBuffer::empty(),
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

impl SendablePacket for RequestAuthGS {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
