use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;

#[derive(Debug)]
pub struct KickPlayer {
    pub buffer: SendablePacketBuffer,
    pub account_name: String,
}

impl KickPlayer {
    pub fn new(account_name: &str) -> Self {
        let mut pack = Self {
            buffer: SendablePacketBuffer::new(),
            account_name: account_name.to_string(),
        };
        pack.write_all().unwrap();
        pack
    }

    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(0x04)?;
        self.buffer.write_string(Some(&self.account_name))?;
        Ok(())
    }
}

impl SendablePacket for KickPlayer {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
