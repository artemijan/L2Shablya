use l2_core::session::SessionKey;
use l2_core::shared_packets::common::LoginServerOpcodes;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use crate::packet::to_client::PlayOk;
    use l2_core::session::SessionKey;
    use l2_core::shared_packets::common::SendablePacket;

    #[test]
    fn test_play_ok() {
        let sk = SessionKey {
            login_ok1: 9,
            login_ok2: 8,
            play_ok1: 7,
            play_ok2: 6,
        };
        let mut packet = PlayOk::new(&sk).unwrap();
        let bytes = packet.buffer.get_data_mut(false);
        assert_eq!(bytes, [11, 0, 7, 7, 0, 0, 0, 6, 0, 0, 0]);
    }
}
