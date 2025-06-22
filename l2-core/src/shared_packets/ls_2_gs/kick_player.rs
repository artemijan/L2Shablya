use bytes::BytesMut;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::{common::ReadablePacket, write::SendablePacketBuffer};

#[derive(Debug, Clone)]
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
        self.buffer
            .write_c_utf16le_string(Some(&self.account_name))?;
        Ok(())
    }
}

impl ReadablePacket for KickPlayer {
    const PACKET_ID: u8 = 0x04;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let account_name = buffer.read_c_utf16le_string()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            account_name,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_kick_player() {
        let acc = "test";
        let mut packet = KickPlayer::new(acc);
        let data = packet.buffer.get_data_mut(false);
        assert_eq!(data, [13, 0, 4, 116, 0, 101, 0, 115, 0, 116, 0, 0, 0]);
    }
    #[test]
    fn test_kick_player_read() {
        let buff = BytesMut::from(&[4, 116, 0, 101, 0, 115, 0, 116, 0, 0, 0][..]);
        let packet = KickPlayer::read(buff).unwrap();
        assert_eq!(packet.account_name, "test");
    }
}
