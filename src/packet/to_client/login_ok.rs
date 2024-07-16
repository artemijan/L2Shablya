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
        let mut login_ok = LoginOk {
            buffer: SendablePacketBuffer::new(),
            login_ok1: session_key.login_ok1,
            login_ok2: session_key.login_ok2,
        };
        login_ok.write_all().unwrap();
        login_ok
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_i8(LoginServerOpcodes::LoginOk as i8)?;
        self.buffer.write_i32(self.login_ok1)?;
        self.buffer.write_i32(self.login_ok2)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_i32(0x000003ea)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_i32(0x00)?;
        self.buffer.write_bytes(vec![0; 16])?;
        Ok(())
    }
}

impl SendablePacket for LoginOk {
    fn get_bytes(&self) -> Vec<u8> {
        self.buffer.get_data()
    }
}
