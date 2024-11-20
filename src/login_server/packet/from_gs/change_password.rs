use crate::login_server::gs_thread::GSHandler;
use crate::common::traits::handlers::PacketHandler;
use crate::common::packet::read::ReadablePacketBuffer;
use crate::login_server::packet::common::GSHandle;
use crate::common::packet::error;
use async_trait::async_trait;
use crate::common::packet::{ReadablePacket, SendablePacket};

#[derive(Clone, Debug)]
pub struct ChangePassword {
    pub account: String,
    pub char_name: String,
    pub current_password: String,
    pub new_password: String,
}

impl ReadablePacket for ChangePassword {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        Some(Self {
            account: buffer.read_string(),
            char_name: buffer.read_string(),
            current_password: buffer.read_string(),
            new_password: buffer.read_string(),
        })
    }
}

#[async_trait]
impl GSHandle for ChangePassword {
    async fn handle(
        &self,
        gs: &mut GSHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        todo!()
    }
}
