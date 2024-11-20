use crate::common::packets::write::SendablePacketBuffer;
use crate::common::packets::SendablePacket;

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
        let _ = gg.write_all();
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x02)?;
        self.buffer.write_u8(self.server_id)?;
        self.buffer.write_string(Some("Bartz"))?; //todo: implement mapping for server names
        Ok(())
    }
}

impl SendablePacket for AuthGS {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
