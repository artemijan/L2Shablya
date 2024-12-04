use async_trait::async_trait;

use crate::{
    common::packets::{
        common::{HandlablePacket, SendablePacket},
        error, ls_2_gs,
    },
    game_server::handlers::LoginHandler,
};

#[async_trait]
impl HandlablePacket for ls_2_gs::AuthGS {
    type HandlerType = LoginHandler;
    async fn handle(
        &self,
        lh: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        print!("Auth response: {:}", self.server_id);
        Ok(None)
    }
}
