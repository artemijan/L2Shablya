use crate::ls_client::LoginServerClient;
use kameo::message::{Context, Message};
use tracing::instrument;
use l2_core::shared_packets::gs_2_ls::PlayerTracert;
use l2_core::traits::ServerToServer;

impl Message<PlayerTracert> for LoginServerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: PlayerTracert,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let _ = self.send_packet(msg).await;
        Ok(())
    }
}
