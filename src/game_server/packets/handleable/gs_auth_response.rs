use async_trait::async_trait;

use crate::common::packets::common::GSStatus;
use crate::common::packets::error::PacketRun;
use crate::common::packets::gs_2_ls::GSStatusUpdate;
use crate::common::packets::write::SendablePacketBuffer;
use crate::common::traits::handlers::PacketHandler;
use crate::{
    common::packets::{common::HandleablePacket, ls_2_gs},
    game_server::handlers::LoginHandler,
};

#[async_trait]
impl HandleablePacket for ls_2_gs::AuthGS {
    type HandlerType = LoginHandler;
    async fn handle(&self, lh: &mut Self::HandlerType) -> Result<(), PacketRun> {
        let cfg = lh.get_controller().get_cfg();
        if self.server_id != cfg.server_id && !cfg.accept_alternative_id {
            return Err(PacketRun {
                msg: Some(format!(
                    "Can not accept alternative id from login server. Id is {}",
                    self.server_id
                )),
            });
        }
        let mut gsu = GSStatusUpdate {
            buffer: SendablePacketBuffer::new(),
            status: if cfg.gm_only {
                GSStatus::GmOnly
            } else {
                GSStatus::Auto
            },
            use_square_brackets: cfg.use_brackets,
            max_players: cfg.max_players,
            server_type: cfg.server_type as i32,
            server_age: cfg.server_age,
        };
        gsu.write_all()?;
        lh.send_packet(Box::new(gsu)).await?;
        println!(
            "Registered on Login server: {:} ({:})",
            self.server_name, self.server_id
        );
        Ok(())
    }
}
