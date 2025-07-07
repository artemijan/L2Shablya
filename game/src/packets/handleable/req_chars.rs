use crate::ls_client::LoginServerClient;
use entities::entities::character;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::{gs_2_ls::ReplyChars, ls_2_gs::RequestChars};
use tracing::instrument;
use l2_core::traits::ServerToServer;

impl Message<RequestChars> for LoginServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RequestChars,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let chars = character::Model::find_by_username(&self.db_pool, &msg.account_name).await?;
        let pack = ReplyChars::new(msg.account_name.clone(), &chars)?;
        self.send_packet(pack).await?;
        Ok(())
    }
}
