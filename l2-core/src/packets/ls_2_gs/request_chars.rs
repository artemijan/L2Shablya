use crate::packets::{
    common::{ReadablePacket, SendablePacket},
    read::ReadablePacketBuffer,
    write::SendablePacketBuffer,
};

#[derive(Debug, Clone)]
pub struct RequestChars {
    pub buffer: SendablePacketBuffer,
    pub account_name: String,
}

impl RequestChars {
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

impl SendablePacket for RequestChars {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
impl ReadablePacket for RequestChars {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let account_name = buffer.read_string();
        Some(Self {
            buffer: SendablePacketBuffer::empty(),
            account_name,
        })
    }
}
