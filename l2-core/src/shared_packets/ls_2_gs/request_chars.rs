use macro_common::SendablePacketImpl;
use crate::shared_packets::{
    common::ReadablePacket,
    read::ReadablePacketBuffer,
    write::SendablePacketBuffer,
};
use crate as l2_core;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct RequestChars {
    pub buffer: SendablePacketBuffer,
    pub account_name: String,
}

impl RequestChars {
    #[must_use]
    pub fn new(account_name: &str) -> RequestChars {
        let mut gg = RequestChars {
            buffer: SendablePacketBuffer::new(),
            account_name: account_name.to_string(),
        };
        let _ = gg.write_all(); // safe to ignore
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x05)?;
        self.buffer.write_string(Some(&self.account_name))?;
        Ok(())
    }
}

impl ReadablePacket for RequestChars {
    const PACKET_ID: u8 = 0x05;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let account_name = buffer.read_string();
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            account_name,
        })
    }
}
