use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::{GSHandle};
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRunError;
use async_trait::async_trait;
use crate::login_server::gs_handler::GSHandler;

#[derive(Clone, Debug)]
pub struct PlayerLogout {
    acc:String
}

impl ReadablePacket for PlayerLogout {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let acc = buffer.read_string();
        Some(Self {
            acc
        })
    }
}

#[async_trait]
impl GSHandle for PlayerLogout {
    async fn handle(
        &self,
        _gs: &mut GSHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRunError> {
        Ok(None)
    }
}
