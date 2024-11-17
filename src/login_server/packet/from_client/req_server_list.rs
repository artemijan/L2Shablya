use crate::login_server::client_thread::ClientHandler;
use crate::common::traits::handler::PacketHandler;
use crate::common::packet::read::ReadablePacketBuffer;
use crate::login_server::packet::common::ClientHandle;
use crate::login_server::packet::to_client::ServerList;
use crate::login_server::packet::{login_fail, PlayerLoginFailReasons};
use async_trait::async_trait;
use crate::common::packet::{error, ReadablePacket, SendablePacket};

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
impl ClientHandle for RequestServerList {
    async fn handle(
        &self,
        ch: &mut ClientHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        if let Some(ref acc_name) = ch.account_name {
            Ok(Some(Box::new(ServerList::new(ch, acc_name))))
        } else {
            Err(error::PacketRun {
                msg: Some(format!("Login Fail, tried user: {:?}", ch.account_name)),
                response: Some(Box::new(login_fail::PlayerLogin::new(
                    PlayerLoginFailReasons::ReasonUserOrPassWrong,
                ))),
            })
        }
    }
}
