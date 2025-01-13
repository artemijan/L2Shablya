use crate::client_thread::ClientHandler;
use crate::packet::to_client::PlayOk;
use crate::packet::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;
use l2_core::shared_packets::error::PacketRun;
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct RequestGSLogin {
    pub s_key_1: i32,
    pub s_key_2: i32,
    pub server_id: u8,
}

impl ReadablePacket for RequestGSLogin {
    const PACKET_ID: u8 = 0x02;

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
        ch.check_session(self.s_key_1, self.s_key_2)?;
        ch.send_packet(Box::new(PlayOk::new(ch.get_session_key())?))
            .await?;
        Ok(())
    }
}
