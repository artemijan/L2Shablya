use crate::client_thread::ClientHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::common::ReadablePacket;

#[derive(Debug, Clone)]
pub struct SendClientIni;

impl ReadablePacket for SendClientIni {
    const PACKET_ID: u8 = 0xD0;
    const EX_PACKET_ID: Option<u16> = Some(0x104);

    fn read(_: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl HandleablePacket for SendClientIni {
    type HandlerType = ClientHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> anyhow::Result<()> {
        //todo: I don't know what this packet is needed for, in L2J it is also not handled
        Ok(())
    }
}
