use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct PlayerTracert {
    pub account: String,
    pub pc_ip: String,
    pub hop1: String,
    pub hop2: String,
    pub hop3: String,
    pub hop4: String,
}

#[async_trait]
impl ReadablePacket for PlayerTracert {
    const PACKET_ID: u8 = 0x07;
const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte();
        let account_name = buffer.read_string();
        let pc_ip = buffer.read_string();
        let hop1 = buffer.read_string();
        let hop2 = buffer.read_string();
        let hop3 = buffer.read_string();
        let hop4 = buffer.read_string();
        Ok(Self {
            account: account_name,
            pc_ip,
            hop1,
            hop2,
            hop3,
            hop4,
        })
    }
}
