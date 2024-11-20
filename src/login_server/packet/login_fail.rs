use super::GSLoginFailReasons;
use crate::common::packets::write::SendablePacketBuffer;
use crate::common::packets::SendablePacket;
use crate::login_server::packet::{LoginServerOpcodes, PlayerLoginFailReasons};

#[derive(Debug)]
pub struct PlayerLogin {
    pub buffer: SendablePacketBuffer,
    pub reason: PlayerLoginFailReasons,
}

#[derive(Debug)]
pub struct GSLogin {
    pub buffer: SendablePacketBuffer,
    pub reason: GSLoginFailReasons,
}

impl GSLogin {
    pub fn new(reason: GSLoginFailReasons) -> GSLogin {
        let mut login_ok = GSLogin {
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

impl PlayerLogin {
    pub fn new(reason: PlayerLoginFailReasons) -> PlayerLogin {
        let mut login_ok = PlayerLogin {
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

impl SendablePacket for PlayerLogin {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
impl SendablePacket for GSLogin {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
