use bytes::BytesMut;
use macro_common::SendablePacket;
use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use crate as l2_core;

#[derive(Clone, Debug, SendablePacket)]
pub struct PlayerLogout {
    pub acc: String,
    pub buffer: SendablePacketBuffer,
}
impl PlayerLogout {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(acc: &str) -> anyhow::Result<Self> {
        let mut inst = Self {
            acc: String::from(acc),
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_c_utf16le_string(Some(acc))?;
        Ok(inst)
    }
}
impl ReadablePacket for PlayerLogout {
    const PACKET_ID: u8 = 0x03;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let acc = buffer.read_c_utf16le_string()?;
        Ok(Self {
            acc,
            buffer: SendablePacketBuffer::empty(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_player_logout() {
        let acc = "test";
        let mut packet = PlayerLogout::new(acc).unwrap();
        let data = packet.buffer.get_data_mut(false);
        assert_eq!(data, [13, 0, 3, 116, 0, 101, 0, 115, 0, 116, 0, 0, 0]);
    }
    #[test]
    fn test_player_logout_read() {
        let buff = [3, 116, 0, 101, 0, 115, 0, 116, 0, 0, 0];
        let packet = PlayerLogout::read(BytesMut::from(&buff[..])).unwrap();
        assert_eq!(packet.acc, "test");
    }
}
