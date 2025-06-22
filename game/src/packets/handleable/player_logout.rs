use crate::ls_client::LoginServerClient;
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::PlayerLogout;
use tracing::instrument;
use l2_core::traits::ServerToServer;

impl Message<PlayerLogout> for LoginServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: PlayerLogout,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let _ = self.send_packet(msg.buffer).await;
        Ok(())
    }
}
