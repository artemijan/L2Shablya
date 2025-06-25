use crate::ls_client::{LSMessages, LoginServerClient};
use kameo::message::{Context, Message};
use l2_core::shared_packets::ls_2_gs::PlayerAuthResponse;
use tracing::instrument;

impl Message<PlayerAuthResponse> for LoginServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: PlayerAuthResponse,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        if let Some(sender) = self.pending_requests.remove(&msg.account) {
            let _ = sender.send(LSMessages::PlayerAuthResponse(msg)); //ignore errors, not critical
        }
        Ok(())
    }
}
