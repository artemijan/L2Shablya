use crate::common::packets::common::{HandlablePacket, PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket, SendablePacket};
use crate::login_server::client_thread::ClientHandler;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::packets::error::PacketRun;
use crate::login_server::packet::to_client::AuthGG;
use async_trait::async_trait;

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
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        if buffer.get_remaining_length() > 20 {
            let session_id = buffer.read_i32();
            let data1 = buffer.read_i32();
            let data2 = buffer.read_i32();
            let data3 = buffer.read_i32();
            let data4 = buffer.read_i32();
            return Some(Self {
                session_id,
                _data1: data1,
                _data2: data2,
                _data3: data3,
                _data4: data4,
            });
        }
        None
    }
}

#[async_trait]
impl HandlablePacket for RequestAuthGG {
    type HandlerType = ClientHandler;
    async fn handle(
        &self,
        ch: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        if self.session_id != ch.get_session_id() {
            return Err(PacketRun {
                msg: Some(format!("Wrong session id {}", self.session_id)),
                response: Some(Box::new(PlayerLoginFail::new(
                    PlayerLoginFailReasons::ReasonAccessFailed,
                ))),
            });
        }
        Ok(Some(Box::new(AuthGG::new(ch.get_session_id()))))
    }
}
