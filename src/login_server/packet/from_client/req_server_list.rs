use crate::common::packets::common::{
    HandleablePacket, PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket,
};
use crate::common::packets::error::PacketRun;
use crate::common::packets::read::ReadablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;
use crate::login_server::client_thread::ClientHandler;
use crate::login_server::packet::to_client::ServerList;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct RequestServerList {
    pub login_ok_1: i32,
    pub login_ok_2: i32,
}

impl ReadablePacket for RequestServerList {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        Some(Self {
            login_ok_1: buffer.read_i32(),
            login_ok_2: buffer.read_i32(),
        })
    }
}

#[async_trait]
impl HandleablePacket for RequestServerList {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> Result<(), PacketRun> {
        if let Some(ref acc_name) = ch.account_name {
            ch.send_packet(Box::new(ServerList::new(ch, acc_name)))
                .await?;
            Ok(())
        } else {
            ch.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonUserOrPassWrong,
            )))
            .await?;
            Err(PacketRun {
                msg: Some(format!("Login Fail, tried user: {:?}", ch.account_name)),
            })
        }
    }
}
