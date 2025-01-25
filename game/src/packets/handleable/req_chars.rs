use l2_core::traits::handlers::{PacketHandler, PacketSender};
use l2_core::shared_packets::{
    ls_2_gs::RequestChars,
    gs_2_ls::ReplyChars,
};
use async_trait::async_trait;
use entities::entities::character;
use tracing::instrument;
use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;

#[async_trait]
impl HandleablePacket for RequestChars {
    type HandlerType = LoginHandler;

    #[instrument(skip_all)]
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let db_pool = gs.get_db_pool();
        let chars =
            character::Model::find_by_username(db_pool, &self.account_name).await?;
        let pack = ReplyChars::new(self.account_name.clone(), &chars);
        gs.send_packet(Box::new(pack)).await?;
        Ok(())
    }
}
