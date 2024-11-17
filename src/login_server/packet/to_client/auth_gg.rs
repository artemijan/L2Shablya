use crate::login_server::packet::common::write::SendablePacketBuffer;
use crate::login_server::packet::common::SendablePacket;
use crate::login_server::packet::LoginServerOpcodes;

#[derive(Debug)]
pub struct AuthGG {
    pub buffer: SendablePacketBuffer,
    session_id: i32,
}

impl AuthGG {
    pub fn new(session_id: i32) -> AuthGG {
        let mut gg = AuthGG {
            buffer: SendablePacketBuffer::new(),
            session_id,
        };
        let _ = gg.write_all();
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_i8(LoginServerOpcodes::GgAuth as i8)?;
        self.buffer.write_i32(self.session_id)?;
        self.buffer.write_i32(0)?;
        self.buffer.write_i32(0)?;
        self.buffer.write_i32(0)?;
        self.buffer.write_i32(0)?;
        Ok(())
    }
}

impl SendablePacket for AuthGG {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
