use l2_core::session::SessionKey;
use l2_core::shared_packets::common::LoginServerOpcodes;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[derive(Debug, SendablePacketImpl)]
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

#[cfg(test)]
mod tests {
    use crate::packet::to_client::LoginOk;
    use l2_core::session::SessionKey;
    use l2_core::shared_packets::common::SendablePacket;

    #[test]
    fn test_login_ok() {
        let sk = SessionKey {
            login_ok1: 9,
            login_ok2: 8,
            play_ok1: 7,
            play_ok2: 6,
        };
        let mut packet = LoginOk::new(&sk);
        let bytes = packet.get_bytes(false);
        assert_eq!(
            bytes,
            [
                51, 0, 3, 9, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 234, 3, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
}
