use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;

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
        gg.write_all().unwrap();
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
