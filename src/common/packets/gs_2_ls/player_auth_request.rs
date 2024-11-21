use crate::common::packets::common::ReadablePacket;
use crate::common::packets::error::PacketRun;
use crate::common::packets::ls_2_gs::{KickPlayer, PlayerAuthResponse};
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::session::SessionKey;
use crate::common::traits::handlers::PacketHandler;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct PlayerAuthRequest {
    pub session: SessionKey,
    pub account_name: String,
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
        Some(Self {
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
