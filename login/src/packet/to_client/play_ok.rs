use l2_core::session::SessionKey;
use l2_core::shared_packets::common::{LoginServerOpcodes, SendablePacket};
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct PlayOk {
    pub buffer: SendablePacketBuffer,
    play_ok1: i32,
    play_ok2: i32,
}

impl PlayOk {
    pub fn new(session_key: &SessionKey) -> anyhow::Result<PlayOk> {
        let mut login_ok = PlayOk {
            buffer: SendablePacketBuffer::new(),
            play_ok1: session_key.play_ok1,
            play_ok2: session_key.play_ok2,
        };
        login_ok.write_all()?;
        Ok(login_ok)
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_i8(LoginServerOpcodes::PlayOk as i8)?;
        self.buffer.write_i32(self.play_ok1)?;
        self.buffer.write_i32(self.play_ok2)?;
        Ok(())
    }
}
