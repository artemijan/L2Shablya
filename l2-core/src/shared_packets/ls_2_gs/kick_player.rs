use macro_common::SendablePacketImpl;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::{
    common::ReadablePacket,
    write::SendablePacketBuffer,
};
use crate as l2_core;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct KickPlayer {
    pub buffer: SendablePacketBuffer,
    pub account_name: String,
}

impl KickPlayer {
    #[must_use]
    pub fn new(account_name: &str) -> Self {
        let mut pack = Self {
            buffer: SendablePacketBuffer::new(),
            account_name: account_name.to_string(),
        };
        let _ = pack.write_all();
        pack
    }

    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(0x04)?;
        self.buffer.write_string(Some(&self.account_name))?;
        Ok(())
    }
}

impl ReadablePacket for KickPlayer {
    const PACKET_ID: u8 = 0x04;
const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let account_name = buffer.read_string()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            account_name,
        })
    }
}
