use crate::gs_client::{GSMessages, GameServerClient};
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::ReplyChars;
use tracing::instrument;

impl Message<ReplyChars> for GameServerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: ReplyChars,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Some(sender) = self.pending_requests.remove(&msg.account_name) {
            let _ = sender.send(GSMessages::ReplyChars(msg)); //ignore errors, not critical
        }
        Ok(())
    }
}
