use async_trait::async_trait;

use crate::common::packets::error::PacketRun;
use crate::{
    common::packets::common::{GSLoginFail, HandleablePacket},
    game_server::handlers::LoginHandler,
};

#[async_trait]
impl HandleablePacket for GSLoginFail {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> Result<(), PacketRun> {
        Err(PacketRun {
            msg: Some(format!(
                "Failed to register on Login server{:?}",
                self.reason
            )),
        })
    }
}
