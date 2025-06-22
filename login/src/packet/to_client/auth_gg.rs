use l2_core::shared_packets::{common::LoginServerOpcodes, write::SendablePacketBuffer};
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

#[cfg(test)]
mod tests {
    use crate::packet::to_client::AuthGG;

    #[test]
    fn test_auth_gg() {
        let mut packet = AuthGG::new(999);
        let bytes = packet.buffer.get_data_mut(false);
        assert_eq!(
            bytes,
            [
                23, 0, 11, 231, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }
}
