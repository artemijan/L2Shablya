use async_trait::async_trait;

use crate::{
    common::packets::{
        common::{HandlablePacket, SendablePacket},
        error,
        gs_2_ls::RequestAuthGS,
        ls_2_gs::{AuthGS, InitLS},
        write::SendablePacketBuffer,
    },
    game_server::handlers::LoginHandler,
};

#[async_trait]
impl HandlablePacket for InitLS {
    type HandlerType = LoginHandler;
    async fn handle(
        &self,
        gs: &mut Self::HandlerType,
    ) -> Result<Option<Box<dyn SendablePacket>>, error::PacketRun> {
        let buffer = SendablePacketBuffer::new();
        let mut ra = RequestAuthGS {
            buffer,
            desired_id: 1,
            accept_alternative_id: false,
            host_reserved: false,
            port: 9014,
            max_players: 5000,
            hex_id: vec![],
            hosts: vec![],
        };
        ra.write_all().expect("Can not build RequestAuthGS packet");
        Ok(Some(Box::new(ra)))
    }
}
