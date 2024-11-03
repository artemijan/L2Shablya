use crate::common::session::SessionKey;
use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;
use crate::packet::LoginServerOpcodes;

#[derive(Debug, Clone)]
pub struct PlayOk {
    pub buffer: SendablePacketBuffer,
    play_ok1: i32,
    play_ok2: i32,
}

impl PlayOk {
    pub fn new(session_key: &SessionKey) -> PlayOk {
        let mut login_ok = PlayOk {
            buffer: SendablePacketBuffer::new(),
            play_ok1: session_key.play_ok1,
            play_ok2: session_key.play_ok2,
        };
        let _ = login_ok.write_all();
        login_ok
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_i8(LoginServerOpcodes::PlayOk as i8)?;
        self.buffer.write_i32(self.play_ok1)?;
        self.buffer.write_i32(self.play_ok2)?;
        Ok(())
    }
}

impl SendablePacket for PlayOk {
    
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
