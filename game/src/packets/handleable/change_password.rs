use crate::ls_client::LoginServerClient;
use kameo::message::Context;
use kameo::prelude::Message;
use tracing::instrument;
use l2_core::shared_packets::gs_2_ls::ChangePassword;

impl Message<ChangePassword> for LoginServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        _: ChangePassword,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo:
        Ok(())
    }
}
