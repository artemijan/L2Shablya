use crate::packets::common::{ReadablePacket, SendablePacket};
use crate::packets::read::ReadablePacketBuffer;
use crate::packets::write::SendablePacketBuffer;

#[derive(Clone, Debug, Default)]
pub struct RequestAuthGSBuilder {
    desired_id: u8,
    accept_alternative_id: bool,
    host_reserved: bool,
    port: u16,
    max_players: u32,
    hex_id: Vec<u8>,
    hosts: Vec<String>,
}

impl RequestAuthGSBuilder {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    pub fn desired_id(mut self, id: u8) -> Self {
        self.desired_id = id;
        self
    }
    pub fn accept_alternative_id(mut self, accept: bool) -> Self {
        self.accept_alternative_id = accept;
        self
    }
    pub fn host_reserved(mut self, reserved: bool) -> Self {
        self.host_reserved = reserved;
        self
    }
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    pub fn max_players(mut self, max_players: u32) -> Self {
        self.max_players = max_players;
        self
    }
    pub fn hex_id(mut self, hex_id: Vec<u8>) -> Self {
        self.hex_id = hex_id;
        self
    }
    pub fn hosts(mut self, hosts: Vec<String>) -> Self {
        self.hosts = hosts;
        self
    }
    pub fn build(self) -> anyhow::Result<RequestAuthGS> {
        let mut inst = RequestAuthGS {
            buffer: SendablePacketBuffer::new(),
            desired_id: self.desired_id,
            accept_alternative_id: self.accept_alternative_id,
            host_reserved: self.host_reserved,
            port: self.port,
            max_players: self.max_players,
            hex_id: self.hex_id,
            hosts: self.hosts,
        };
        inst.write_all()?;
        Ok(inst)
    }
}

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
    pub fn builder() -> RequestAuthGSBuilder {
        RequestAuthGSBuilder::new()
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
