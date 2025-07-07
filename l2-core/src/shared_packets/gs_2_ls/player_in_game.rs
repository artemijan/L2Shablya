use bytes::BytesMut;
use macro_common::SendablePacket;
use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use crate as l2_core;

#[derive(Clone, Debug, SendablePacket)]
pub struct PlayerInGame {
    pub buffer: SendablePacketBuffer,
    pub accounts: Vec<String>,
}

impl PlayerInGame {
    ///
    /// # Errors
    /// - when packet size is too big
    pub fn new(accounts: &[String]) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            accounts: accounts.to_vec(),
        };
        inst.write_all()?;
        Ok(inst)
    }
    /// # Errors
    /// - when packet size is too big
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    pub fn write_all(&mut self) -> anyhow::Result<()> {
        self.buffer.write(0x02)?;
        self.buffer.write_i16(self.accounts.len() as i16)?;
        for acc in &self.accounts {
            self.buffer.write_c_utf16le_string(Some(acc))?;
        }
        Ok(())
    }
}

impl ReadablePacket for PlayerInGame {
    const PACKET_ID: u8 = 0x02;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let size = buffer.read_i16()?;
        let mut accounts: Vec<String> = vec![];
        for _ in 0..size {
            let st = buffer.read_c_utf16le_string()?;
            accounts.push(st);
        }
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            accounts,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::shared_packets::common::{ReadablePacket, SendablePacket};

    #[test]
    fn test_player_in_game_new() {
        let accounts = vec!["admin".to_string(), "adm".to_string()];
        let packet = PlayerInGame::new(&accounts).unwrap();
        let data = packet.buffer.take().to_vec();
        assert_eq!(
            data,
            [
                25, 0, 2, 2, 0, 97, 0, 100, 0, 109, 0, 105, 0, 110, 0, 0, 0, 97, 0, 100, 0, 109, 0,
                0, 0
            ]
        );
    }
    #[test]
    fn test_player_in_game_read() {
        let expected_accounts = vec!["admin".to_string(), "adm".to_string()];
        let packet = PlayerInGame::read(BytesMut::from(&[
            2, 2, 0, 97, 0, 100, 0, 109, 0, 105, 0, 110, 0, 0, 0, 97, 0, 100, 0, 109, 0, 0, 0,
        ][..]))
        .unwrap();
        assert_eq!(packet.accounts, expected_accounts);
    }
}
