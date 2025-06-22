use crate::gs_client::GameServerClient;
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::ChangePassword;
use tracing::instrument;

impl Message<ChangePassword> for GameServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: ChangePassword,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        todo!()
    }
}
