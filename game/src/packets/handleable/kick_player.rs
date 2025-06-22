use crate::ls_client::LoginServerClient;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::ls_2_gs::KickPlayer;
use tracing::instrument;

impl Message<KickPlayer> for LoginServerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: KickPlayer,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        self.controller.logout_account(&msg.account_name);
        //todo: disconnect TCP client
        Ok(())
    }
}
