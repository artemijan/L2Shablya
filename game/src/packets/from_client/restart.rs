use crate::packets::to_client::{CharSelectionInfo, DeleteObject, RestartResponse};
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
        self.stop_movement();
        let session_id = self.try_get_session_key()?.get_play_session_id();
        let chars = self.try_get_account_chars()?;
        let user_name = self.try_get_user()?.username.clone();
        let p = CharSelectionInfo::new(&user_name.clone(), session_id, &self.controller, chars)?;
        let player = self.try_get_selected_char()?.clone(); // we clone it to avoid borrow checker issues with reference to self
        self.controller.broadcast_packet_with_filter(
            DeleteObject::new(player.get_object_id())?,
            Some(Box::new(move |acc, _| !acc.eq(&user_name))),
        );
        self.send_packet(RestartResponse::ok()?).await?;
        character::Model::update_char(&self.db_pool, &player.char_model).await?;
        self.send_packet(p).await?;
        Ok(())
    }
}
