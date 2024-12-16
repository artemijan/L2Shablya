use std::fmt::Debug;
use async_trait::async_trait;
use l2_core::packets::error::PacketRun;

pub mod handleable;

#[async_trait]
pub trait HandleablePacket: Debug + Send {
    type HandlerType;
    async fn handle(
        &self,
        ch: &mut Self::HandlerType,
    ) -> Result<(), PacketRun>;
}