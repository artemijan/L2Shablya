use crate::ls_client::LoginServerClient;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::gs_2_ls::PlayerInGame;
use l2_core::traits::ServerToServer;
use tracing::instrument;

impl Message<PlayerInGame> for LoginServerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: PlayerInGame,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let _ = self.send_packet(msg).await;
        Ok(())
    }
}
