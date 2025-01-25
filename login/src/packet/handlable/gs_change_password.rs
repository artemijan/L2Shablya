use crate::gs_thread::GSHandler;
use async_trait::async_trait;
use l2_core::shared_packets::gs_2_ls::ChangePassword;
use crate::packet::HandleablePacket;

#[async_trait]
impl HandleablePacket for ChangePassword {
    type HandlerType = GSHandler;
    async fn handle(&self, _: &mut Self::HandlerType) -> anyhow::Result<()> {
        todo!()
    }
}
