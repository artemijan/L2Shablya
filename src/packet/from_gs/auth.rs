use crate::common::dto::game_server::GSInfo;
use crate::login_server::gs_handler::{GSConnectionState, GSHandler};
use crate::login_server::PacketHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::GSHandle;
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use crate::packet::login_fail::PlayerLogin;
use crate::packet::to_gs::AuthGS;
use crate::packet::PlayerLoginFailReasons;
use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct GS {
    desired_id: u8,
    accept_alternative_id: bool,
    host_reserved: bool,
    port: u16,
    max_players: u32,
    hex_id: Vec<u8>,
    hosts: Vec<String>,
}
#[allow(clippy::cast_sign_loss)]
impl ReadablePacket for GS {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte();
        let desired_id = buffer.read_byte();
        let accept_alternative_id = buffer.read_byte() != 0;
        let host_reserved = buffer.read_byte() != 0;
        let port = buffer.read_i16() as u16;
        let max_players = buffer.read_i32() as u32;
        let mut size = buffer.read_i32();
        let hex_id = buffer.read_bytes(size as usize);
        size = buffer.read_i32() * 2;
        let hosts = buffer.read_n_strings(size as usize);
        Some(GS {
            desired_id,
            accept_alternative_id,
            host_reserved,
            port,
            max_players,
            hex_id,
            hosts,
        })
    }
}

#[async_trait]
impl GSHandle for GS {
    async fn handle(&self, gs: &mut GSHandler) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        let gsi = GSInfo {
            id: self.desired_id,
            accept_alternative_id: self.accept_alternative_id,
            host_reserved: self.host_reserved,
            port: self.port,
            is_authed: true,
            status: 0,
            is_pvp: true,
            server_type: 0,
            age_limit: 0,
            show_brackets: false,
            max_players: self.max_players,
            hex_id: self.hex_id.clone(),
            hosts: self.hosts.clone(),
        };
        match gs.get_lc().register_gs(gsi).await {
            Ok(()) => {
                gs.connection_state.transition_to(&GSConnectionState::Authed)?;
                gs.server_id = Some(self.desired_id);
                Ok(Some(Box::new(AuthGS::new(self.desired_id))))
            }
            Err(e) => Err(e),
        }
    }
}
