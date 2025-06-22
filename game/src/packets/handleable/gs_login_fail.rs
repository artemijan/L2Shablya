use crate::ls_client::LoginServerClient;
use anyhow::bail;
use kameo::message::{Context, Message};
use l2_core::shared_packets::common::GSLoginFail;
use tracing::instrument;

impl Message<GSLoginFail> for LoginServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: GSLoginFail,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        bail!("Failed to register on Login server{:?}", msg.reason)
    }
}
