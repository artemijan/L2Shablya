use crate::client_thread::ClientHandler;
use crate::packet::to_client::AuthGG;
use crate::packet::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

#[derive(Clone, Debug)]
pub struct RequestAuthGG {
    pub session_id: i32,
    ///unused data from the packet
    _data1: i32,
    _data2: i32,
    _data3: i32,
    _data4: i32,
}

impl ReadablePacket for RequestAuthGG {
    const PACKET_ID: u8 = 0x07;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        if buffer.get_remaining_length() > 20 {
            let session_id = buffer.read_i32()?;
            let data1 = buffer.read_i32()?;
            let data2 = buffer.read_i32()?;
            let data3 = buffer.read_i32()?;
            let data4 = buffer.read_i32()?;
            return Ok(Self {
                session_id,
                _data1: data1,
                _data2: data2,
                _data3: data3,
                _data4: data4,
            });
        }
        bail!("Not enough data to read packet");
    }
}

#[async_trait]
impl HandleablePacket for RequestAuthGG {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> anyhow::Result<()> {
        if self.session_id != ch.get_session_id() {
            ch.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonAccessFailed,
            )?))
            .await?;
            bail!(format!("Wrong session id {}", self.session_id));
        }
        ch.send_packet(Box::new(AuthGG::new(ch.get_session_id())))
            .await?;
        Ok(())
    }
}
