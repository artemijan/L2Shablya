use std::fmt::Debug;
use async_trait::async_trait;
use l2_core::packets::error::PacketRun;

pub mod handleable;
pub mod from_client;
pub mod to_client;

#[async_trait]
pub trait HandleablePacket: Debug + Send {
    type HandlerType;
    async fn handle(
        &self,
        ch: &mut Self::HandlerType,
    ) -> Result<(), PacketRun>;
}