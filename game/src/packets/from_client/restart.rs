use crate::packets::to_client::{CharSelectionInfo, RestartResponse};
use crate::pl_client::PlayerClient;
use bytes::BytesMut;
use entities::entities::character;
use kameo::message::Context;
use kameo::prelude::Message;
use l2_core::shared_packets::common::ReadablePacket;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct RequestRestart {}

impl ReadablePacket for RequestRestart {
    const PACKET_ID: u8 = 0x57;
    const EX_PACKET_ID: Option<u16> = None;
    fn read(_: BytesMut) -> anyhow::Result<Self> {
        Ok(Self {})
    }
}
impl Message<RequestRestart> for PlayerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: RequestRestart,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo: if can logout (olymp, pvp flag, events, etc.)
        self.send_packet(RestartResponse::ok()?).await?;
        let sk = self.try_get_session_key()?;
        let chars = self.try_get_account_chars()?;
        let user_name = &self.try_get_user()?.username;
        let player = self.try_get_selected_char()?;
        character::Model::update_char(&self.db_pool, &player.char_model).await?;
        let p =
            CharSelectionInfo::new(user_name, sk.get_play_session_id(), &self.controller, chars)?;
        self.send_packet(p).await?;
        Ok(())
    }
}
