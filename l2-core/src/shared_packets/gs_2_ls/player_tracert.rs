use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use async_trait::async_trait;
use bytes::BytesMut;

#[derive(Clone, Debug)]
pub struct PlayerTracert {
    pub account: String,
    pub pc_ip: String,
    pub hop1: String,
    pub hop2: String,
    pub hop3: String,
    pub hop4: String,
    pub buffer: SendablePacketBuffer,
}
impl PlayerTracert {
    /// # Errors
    /// - when write packet bytes fails
    pub fn new(
        account: String,
        pc_ip: String,
        hop1: String,
        hop2: String,
        hop3: String,
        hop4: String,
    ) -> anyhow::Result<Self> {
        let mut inst = Self {
            account,
            pc_ip,
            hop1,
            hop2,
            hop3,
            hop4,
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_c_utf16le_string(Some(&inst.account))?;
        inst.buffer.write_c_utf16le_string(Some(&inst.pc_ip))?;
        inst.buffer.write_c_utf16le_string(Some(&inst.hop1))?;
        inst.buffer.write_c_utf16le_string(Some(&inst.hop2))?;
        inst.buffer.write_c_utf16le_string(Some(&inst.hop3))?;
        inst.buffer.write_c_utf16le_string(Some(&inst.hop4))?;
        Ok(inst)
    }
}

#[async_trait]
impl ReadablePacket for PlayerTracert {
    const PACKET_ID: u8 = 0x07;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let account_name = buffer.read_c_utf16le_string()?;
        let pc_ip = buffer.read_c_utf16le_string()?;
        let hop1 = buffer.read_c_utf16le_string()?;
        let hop2 = buffer.read_c_utf16le_string()?;
        let hop3 = buffer.read_c_utf16le_string()?;
        let hop4 = buffer.read_c_utf16le_string()?;
        Ok(Self {
            account: account_name,
            buffer: SendablePacketBuffer::empty(),
            pc_ip,
            hop1,
            hop2,
            hop3,
            hop4,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_player_tracert() {
        let acc_name = [116, 0, 101, 0, 115, 0, 116, 0, 0, 0];
        let pc_ip = b"1\x009\x002\x00.\x001\x006\x008\x00.\x000\x00.\x001\x000\x000\x00\x00\x00";
        let hop1 = b"h\x00o\x00p\x001\x00\x00\x00";
        let hop2 = b"h\x00o\x00p\x002\x00\x00\x00";
        let hop3 = b"h\x00o\x00p\x003\x00\x00\x00";
        let hop4 = b"h\x00o\x00p\x004\x00\x00\x00";
        let mut data = vec![0x07];
        data.extend(&acc_name);
        data.extend(pc_ip);
        data.extend(hop1);
        data.extend(hop2);
        data.extend(hop3);
        data.extend(hop4);
        let packet = PlayerTracert::read(BytesMut::from(&data[..])).unwrap();
        assert_eq!(packet.account, "test");
        assert_eq!(packet.pc_ip, "192.168.0.100");
        assert_eq!(packet.hop1, "hop1");
        assert_eq!(packet.hop2, "hop2");
        assert_eq!(packet.hop3, "hop3");
        assert_eq!(packet.hop4, "hop4");
    }
}
