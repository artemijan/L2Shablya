use crate::login_server::gs_thread::GSHandler;
use crate::login_server::traits::PacketHandler;
use crate::login_server::packet::common::read::ReadablePacketBuffer;
use crate::login_server::packet::common::GSHandle;
use crate::login_server::packet::common::{ReadablePacket, SendablePacket};
use crate::login_server::packet::error::PacketRun;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct PlayerLogout {
    acc: String,
}

impl ReadablePacket for PlayerLogout {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let acc = buffer.read_string();
        Some(Self { acc })
    }
}

#[async_trait]
impl GSHandle for PlayerLogout {
    async fn handle(
        &self,
        gs: &mut GSHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        let lc = gs.get_lc();
        lc.on_player_logout(&self.acc);
        Ok(None)
    }
}
