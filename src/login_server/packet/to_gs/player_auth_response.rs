use crate::common::packet::write::SendablePacketBuffer;
use crate::common::packet::SendablePacket;

#[derive(Debug)]
pub struct PlayerAuthResponse {
    pub buffer: SendablePacketBuffer,
    account: String,
    is_ok: bool,
}

impl PlayerAuthResponse {
    pub fn new(account: &str, is_ok: bool) -> PlayerAuthResponse {
        let mut gg = PlayerAuthResponse {
            buffer: SendablePacketBuffer::new(),
            account: account.to_string(),
            is_ok,
        };
        let _ = gg.write_all();
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x03)?;
        self.buffer.write_string(Some(&self.account))?;
        self.buffer.write_u8(u8::from(self.is_ok))?;
        Ok(())
    }
}

impl SendablePacket for PlayerAuthResponse {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
