use crate::ls_client::{LSMessages, LoginServerClient};
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::PlayerAuthRequest;
use tokio::sync::oneshot;
use tracing::instrument;
use l2_core::traits::ServerToServer;

impl Message<PlayerAuthRequest> for LoginServerClient {
    type Reply = anyhow::Result<oneshot::Receiver<LSMessages>>;
    #[instrument(skip(self,_ctx))]
    async fn handle(
        &mut self,
        msg: PlayerAuthRequest,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<oneshot::Receiver<LSMessages>> {
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(msg.account_name.clone(), tx);
        //we send a packet and immediately return the receiver to unblock the actor,
        // so it can handle the response packet
        self.send_packet(msg.buffer).await?;
        Ok(rx)
    }
}
