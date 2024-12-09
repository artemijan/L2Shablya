use crate::common::packets::common::ReadablePacket;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;

#[derive(Clone, Debug)]
pub struct PlayerInGame {
    pub accounts: Vec<String>,
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
        Some(Self { accounts })
    }
}
