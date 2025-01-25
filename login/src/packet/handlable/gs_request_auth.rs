use crate::packet::HandleablePacket;
use crate::{
    dto::game_server::GSInfo,
    gs_thread::{enums, GSHandler},
};
use anyhow::bail;
use async_trait::async_trait;
use l2_core::constants::get_server_name_by_id;
use l2_core::traits::handlers::PacketSender;
use l2_core::{
    shared_packets::{
        common::{GSLoginFail, GSLoginFailReasons},
        gs_2_ls::RequestAuthGS,
        ls_2_gs::AuthGS,
    },
    traits::handlers::PacketHandler,
};

#[async_trait]
impl HandleablePacket for RequestAuthGS {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let gsi = GSInfo::new(
            self.desired_id,
            self.accept_alternative_id,
            self.host_reserved,
            self.port,
            true,
            0,
            true,
            0,
            0,
            false,
            self.max_players,
            self.hex_id.clone(),
            &self.hosts,
        )?;
        match gs.get_controller().register_gs(gsi) {
            Ok(desired_id) => {
                gs.set_connection_state(&enums::GS::Authed).await?;
                let server_name = get_server_name_by_id(desired_id);
                if let Some(server_name) = server_name {
                    gs.server_id = Some(desired_id);
                    gs.send_packet(Box::new(AuthGS::new(desired_id, server_name)))
                        .await?;
                } else {
                    gs.send_packet(Box::new(GSLoginFail::new(GSLoginFailReasons::None)))
                        .await?;
                }
                Ok(())
            }
            Err(e) => {
                let err_msg = format!(
                    "Failed to register game server with id {:}, fail reason {:?}",
                    self.desired_id, e
                );
                gs.send_packet(Box::new(GSLoginFail::new(e))).await?;
                bail!(err_msg)
            }
        }
    }
}
