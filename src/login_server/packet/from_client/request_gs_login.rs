use crate::login_server::client_thread::ClientHandler;
use crate::common::traits::handlers::PacketHandler;
use crate::common::packet::read::ReadablePacketBuffer;
use crate::login_server::packet::common::ClientHandle;
use crate::common::packet::error::PacketRun;
use crate::login_server::packet::to_client::PlayOk;
use async_trait::async_trait;
use crate::common::packet::{ReadablePacket, SendablePacket};

#[derive(Clone, Debug)]
pub struct RequestGSLogin {
    pub s_key_1: i32,
    pub s_key_2: i32,
    pub server_id: u8,
}

impl ReadablePacket for RequestGSLogin {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        Some(Self {
            s_key_1: buffer.read_i32(),
            s_key_2: buffer.read_i32(),
            server_id: buffer.read_byte(),
        })
    }
}

#[async_trait]
impl ClientHandle for RequestGSLogin {
    async fn handle(
        &self,
        ch: &mut ClientHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        Ok(Some(Box::new(PlayOk::new(ch.get_session_key()))))
    }
}
