use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;
use crate as l2_core;

#[derive(Clone, Debug, SendablePacketImpl)]
pub struct PlayerInGame {
    buffer: SendablePacketBuffer,
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

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let size = buffer.read_i16()?;
        let mut accounts: Vec<String> = vec![];
        for _ in 0..size {
            let st = buffer.read_string()?;
            accounts.push(st);
        }
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            accounts,
        })
    }
}
