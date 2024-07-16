use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;
use crate::packet::{LoginFailReasons, LoginServerOpcodes};

#[derive(Debug)]
pub struct LoginFail {
    pub buffer: SendablePacketBuffer,
    pub reason: LoginFailReasons,
}

impl LoginFail {
    pub fn new(reason: LoginFailReasons) -> LoginFail {
        let mut login_ok = LoginFail {
            buffer: SendablePacketBuffer::new(),
            reason,
        };
        login_ok.write_all().unwrap();
        login_ok
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(LoginServerOpcodes::LoginFail as u8)?;
        self.buffer.write(self.reason.clone() as u8)?;
        Ok(())
    }
}

impl SendablePacket for LoginFail {
    fn get_bytes(&self) -> Vec<u8> {
        self.buffer.get_data()
    }
}
