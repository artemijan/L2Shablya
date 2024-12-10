use crate::common::packets::common::{ReadablePacket, SendablePacket};
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::packets::write::SendablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;

#[derive(Clone, Debug)]
pub struct PlayerInGame {
    buffer: SendablePacketBuffer,
    pub accounts: Vec<String>,
}

impl PlayerInGame {
    pub fn new(accounts: Vec<String>) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            accounts,
        };
        inst.write_all()?;
        Ok(inst)
    }
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
