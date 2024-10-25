use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;

#[derive(Debug)]
pub struct AuthGS {
    pub buffer: SendablePacketBuffer,
    server_id: u8,
}

impl AuthGS {
    pub fn new(server_id: u8) -> AuthGS {
        let mut gg = AuthGS {
            buffer: SendablePacketBuffer::new(),
            server_id,
        };
        gg.write_all().unwrap();
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x02)?;
        self.buffer.write_u8(self.server_id)?;
        self.buffer.write_string(Some("Bartz"))?;
        Ok(())
    }
}

impl SendablePacket for AuthGS {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
