use crate::packets::common::{ReadablePacket, SendablePacket};
use crate::packets::read::ReadablePacketBuffer;
use crate::packets::write::SendablePacketBuffer;

#[derive(Clone, Debug)]
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
            self.buffer.write_string(Some(acc))?;
        }
        Ok(())
    }
}

impl ReadablePacket for PlayerInGame {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let size = buffer.read_i16();
        let mut accounts: Vec<String> = vec![];
        for _ in 0..size {
            let st = buffer.read_string();
            accounts.push(st);
        }
        Some(Self {
            buffer: SendablePacketBuffer::empty(),
            accounts,
        })
    }
}

impl SendablePacket for PlayerInGame {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
