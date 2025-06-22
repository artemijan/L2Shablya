use crate::gs_client::{GSMessages, GameServerClient};
use kameo::message::{Context, Message};
use l2_core::shared_packets::ls_2_gs::RequestChars;
use tokio::sync::oneshot;
use tracing::instrument;
use l2_core::traits::ServerToServer;

impl Message<RequestChars> for GameServerClient {
    type Reply = anyhow::Result<oneshot::Receiver<GSMessages>>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RequestChars,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let (tx, rx) = oneshot::channel();
        self.pending_requests.insert(msg.account_name.clone(), tx);
        //we send a packet and immediately return the receiver to unblock the actor,
        // so it can handle the response packet
        self.send_packet(msg.buffer).await?;
        Ok(rx)
    }
}
