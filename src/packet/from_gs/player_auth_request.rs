use crate::common::session::SessionKey;
use crate::login_server::gs_handler::GSHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::GSHandle;
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use crate::packet::to_gs::PlayerAuthResponse;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct PlayerAuthRequest {
    session: SessionKey,
    account_name: String,
}

impl ReadablePacket for PlayerAuthRequest {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let account_name = buffer.read_string();
        let play_ok1 = buffer.read_i32();
        let play_ok2 = buffer.read_i32();
        let login_ok1 = buffer.read_i32();
        let login_ok2 = buffer.read_i32();
        Some(PlayerAuthRequest {
            account_name,
            session: SessionKey {
                play_ok1,
                play_ok2,
                login_ok1,
                login_ok2,
            },
        })
    }
}

#[async_trait]
impl GSHandle for PlayerAuthRequest {
    async fn handle(&self, _: &mut GSHandler) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        Ok(Some(Box::new(PlayerAuthResponse::new(&self.account_name, true))))
    }
}
