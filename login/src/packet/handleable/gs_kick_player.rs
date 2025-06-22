use crate::gs_client::GameServerClient;
use kameo::message::{Context, Message};
use l2_core::shared_packets::ls_2_gs::KickPlayer;
use tracing::instrument;

impl Message<KickPlayer> for GameServerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: KickPlayer,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.lc.remove_player(&msg.account_name);
        Ok(())
    }
}
