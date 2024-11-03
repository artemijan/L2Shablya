use crate::common::session::SessionKey;
use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;
use crate::packet::LoginServerOpcodes;

#[derive(Debug)]
pub struct LoginOk {
    pub buffer: SendablePacketBuffer,
    login_ok1: i32,
    login_ok2: i32,
}

impl LoginOk {
    pub fn new(session_key: &SessionKey) -> LoginOk {
        let mut login_ok = Self {
            buffer: SendablePacketBuffer::new(),
            login_ok1: session_key.login_ok1,
            login_ok2: session_key.login_ok2,
        };
        // it is safe to unwrap result, as we know hoe many bytes are written
        let _ = login_ok.write_all();
        login_ok
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_i8(LoginServerOpcodes::LoginOk as i8)?;
        self.buffer.write_i32(self.login_ok1)?;
        self.buffer.write_i32(self.login_ok2)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_i32(0x0000_03ea)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_bytes(vec![0; 16].as_slice())?;
        Ok(())
    }
}

impl SendablePacket for LoginOk {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
