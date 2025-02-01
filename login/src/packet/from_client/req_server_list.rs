use crate::client_thread::ClientHandler;
use crate::packet::to_client::ServerList;
use crate::packet::HandleablePacket;
use anyhow::bail;
use async_trait::async_trait;
use l2_core::shared_packets::common::{PlayerLoginFail, PlayerLoginFailReasons, ReadablePacket};
use l2_core::shared_packets::read::ReadablePacketBuffer;
use l2_core::traits::handlers::PacketSender;

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct RequestServerList {
    pub login_ok_1: i32,
    pub login_ok_2: i32,
}

impl ReadablePacket for RequestServerList {
    const PACKET_ID: u8 = 0x05;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        Ok(Self {
            login_ok_1: buffer.read_i32()?,
            login_ok_2: buffer.read_i32()?,
        })
    }
}

#[async_trait]
impl HandleablePacket for RequestServerList {
    type HandlerType = ClientHandler;
    async fn handle(&self, ch: &mut Self::HandlerType) -> anyhow::Result<()> {
        if let Some(ref acc_name) = ch.account_name {
            ch.send_packet(Box::new(ServerList::new(ch, acc_name)))
                .await?;
            Ok(())
        } else {
            ch.send_packet(Box::new(PlayerLoginFail::new(
                PlayerLoginFailReasons::ReasonUserOrPassWrong,
            )?))
            .await?;
            bail!(format!("Login Fail, tried user: {:?}", ch.account_name));
        }
    }
}
