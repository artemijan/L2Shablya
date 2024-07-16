use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;

#[derive(Debug)]
pub struct RequestChars {
    pub buffer: SendablePacketBuffer,
    account_name: String,
}

impl RequestChars {
    pub fn new(account_name: &str) -> RequestChars {
        let mut gg = RequestChars {
            buffer: SendablePacketBuffer::new(),
            account_name: account_name.to_string(),
        };
        gg.write_all().unwrap();
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x05)?;
        self.buffer.write_string(Some(&self.account_name))?;
        Ok(())
    }
}

impl SendablePacket for RequestChars {
    fn get_bytes(&self) -> Vec<u8> {
        self.buffer.get_data()
    }
}
