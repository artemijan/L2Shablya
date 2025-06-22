use crate::gs_client::GameServerClient;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::gs_2_ls::PlayerTracert;
use tracing::{info, instrument};

impl Message<PlayerTracert> for GameServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: PlayerTracert,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        //todo: implement me
        info!("TODO: PlayerTracert packet ignored");
        Ok(())
    }
}
