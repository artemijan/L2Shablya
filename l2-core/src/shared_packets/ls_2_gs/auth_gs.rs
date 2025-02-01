use macro_common::SendablePacketImpl;
use crate::shared_packets::{
    common::ReadablePacket,
    read::ReadablePacketBuffer,
    write::SendablePacketBuffer,
};
use crate as l2_core;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct AuthGS {
    buffer: SendablePacketBuffer,
    pub server_id: u8,
    pub server_name: String,
}

impl AuthGS {
    #[must_use]
    pub fn new(server_id: u8, server_name: String) -> AuthGS {
        let mut gg = AuthGS {
            buffer: SendablePacketBuffer::new(),
            server_id,
            server_name,
        };
        let _ = gg.write_all();
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x02)?;
        self.buffer.write_u8(self.server_id)?;
        self.buffer.write_string(Some(&self.server_name))?;
        Ok(())
    }
}

impl ReadablePacket for AuthGS {
    const PACKET_ID: u8 = 0x02;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let _packet_id = buffer.read_byte();
        let server_id = buffer.read_byte();
        let server_name = buffer.read_string();
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            server_id,
            server_name,
        })
    }
}
