use crate::common::packets::common::{HandleablePacket, ReadablePacket};
use crate::common::packets::error::PacketRun;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;
use crate::login_server::client_thread::ClientHandler;
use crate::login_server::packet::to_client::PlayOk;
use async_trait::async_trait;

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
impl HandleablePacket for RequestGSLogin {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> Result<(), PacketRun> {
        ch.send_packet(Box::new(PlayOk::new(ch.get_session_key())))
            .await?;
        Ok(())
    }
}
