use crate::packets::to_client::{CharSelectionInfo, DeleteObject, RestartResponse};
use crate::pl_client::{ClientStatus, PlayerClient};
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
        _msg: RequestRestart,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        //todo: if can logout (olymp, pvp flag, events, etc.)
        let session_id = self.try_get_session_key()?.get_play_session_id();
        let user_name = self.try_get_user()?.username.clone();

        self.stop_movement();
        self.selected_target = None;

        let (selected_slot, updated_char_model) = {
            let selected_slot = self
                .selected_char
                .ok_or_else(|| anyhow::anyhow!("No character selected"))?;
            let mut player = self.try_get_selected_char()?.clone();
            player.char_model.x = player.get_x();
            player.char_model.y = player.get_y();
            player.char_model.z = player.get_z();
            let updated_model =
                character::Model::update_char(&self.db_pool, &player.char_model).await?;
            (selected_slot, updated_model)
        };
        
        self.with_char_by_slot_id(
            selected_slot,
            |_character| async move { Ok(updated_char_model) },
        )
        .await?;

        let player_obj_id = self.try_get_selected_char()?.get_object_id();
        let chars = self.try_get_account_chars()?;

        let p = CharSelectionInfo::new(&user_name, session_id, &self.controller, chars)?;

        self.set_status(ClientStatus::Authenticated);
        self.selected_char = None;

        self.controller.broadcast_packet_with_filter(
            DeleteObject::new(player_obj_id)?,
            Some(Box::new(move |acc, _| !acc.eq(&user_name))),
        );
        self.send_packet(RestartResponse::ok()?).await?;
        self.send_packet(p).await?;
        Ok(())
    }
}
