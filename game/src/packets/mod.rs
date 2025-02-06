use std::fmt::Debug;
use async_trait::async_trait;

pub mod handleable;
pub mod from_client;
pub mod to_client;
pub mod enums;
pub mod utils;

#[async_trait]
pub trait HandleablePacket: Debug + Send {
    type HandlerType;
    async fn handle(
        &self,
        ch: &mut Self::HandlerType,
    ) -> anyhow::Result<()>;
}