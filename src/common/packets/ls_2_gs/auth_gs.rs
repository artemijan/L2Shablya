use crate::common::packets::{
    common::{ReadablePacket, SendablePacket},
    read::ReadablePacketBuffer,
    write::SendablePacketBuffer,
};

#[derive(Debug)]
pub struct AuthGS {
    buffer: SendablePacketBuffer,
    pub server_id: u8,
    pub server_name: String,
}

impl AuthGS {
    pub fn new(server_id: u8, server_name: String) -> AuthGS {
        let mut gg = AuthGS {
            buffer: SendablePacketBuffer::new(),
            server_id,
            server_name
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

impl SendablePacket for AuthGS {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}

impl ReadablePacket for AuthGS {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        let packet_id = buffer.read_byte();
        let server_id = buffer.read_byte();
        let server_name = buffer.read_string();
        Some(Self {
            buffer: SendablePacketBuffer::empty(),
            server_id,
            server_name,
        })
    }
}
