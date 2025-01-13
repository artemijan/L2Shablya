use std::fmt::Debug;
use async_trait::async_trait;
use l2_core::shared_packets::error::PacketRun;

pub mod from_client;
pub mod handlable;
pub mod gs_factory;
pub mod cp_factory;
pub mod to_client;


#[async_trait]
pub trait HandleablePacket: Debug + Send {
    type HandlerType;
    async fn handle(
        &self,
        ch: &mut Self::HandlerType,
    ) -> Result<(), PacketRun>;
}