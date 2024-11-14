use crate::login_server::gs_thread::GSHandler;
use crate::login_server::traits::PacketHandler;
use crate::packet::common::read::ReadablePacketBuffer;
use crate::packet::common::GSHandle;
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use crate::packet::login_fail::PlayerLogin;
use crate::packet::PlayerLoginFailReasons;
use async_trait::async_trait;

#[repr(i32)]
#[derive(Clone, Debug, Default)]
pub enum GSStatus {
    Auto = 0x00,
    Good = 0x01,
    Normal = 0x02,
    Full = 0x03,
    #[default]
    Down = 0x04,
    GmOnly = 0x05,
}

impl GSStatus {
    pub fn from_opcode(opcode: i32) -> Option<Self> {
        match opcode {
            0x00 => Some(Self::Auto),
            0x01 => Some(Self::Good),
            0x02 => Some(Self::Normal),
            0x03 => Some(Self::Full),
            0x04 => Some(Self::Down),
            0x05 => Some(Self::GmOnly),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct GSStatusUpdate {
    pub status: GSStatus,
    pub use_square_brackets: bool,
    pub max_players: u32,
    pub server_type: i32,
    pub server_age: u8,
}

impl GSStatusUpdate {
    const SERVER_LIST_STATUS: i32 = 0x01;
    const SERVER_TYPE: i32 = 0x02;
    const SERVER_LIST_SQUARE_BRACKET: i32 = 0x03;
    const MAX_PLAYERS: i32 = 0x04;
    const TEST_SERVER: i32 = 0x05;
    const SERVER_AGE: i32 = 0x06;
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
impl ReadablePacket for GSStatusUpdate {
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        buffer.read_byte(); //packet id
        let size = buffer.read_i32() as usize;
        let mut instance = Self::default();
        for _ in 0..size {
            let gs_type = buffer.read_i32();
            let value = buffer.read_i32();

            match gs_type {
                Self::SERVER_LIST_STATUS => {
                    if let Some(stat) = GSStatus::from_opcode(value) {
                        instance.status = stat;
                    }
                }
                Self::SERVER_LIST_SQUARE_BRACKET => {
                    instance.use_square_brackets = value != 0;
                }
                Self::MAX_PLAYERS => {
                    instance.max_players = value as u32;
                }
                Self::SERVER_TYPE => {
                    instance.server_type = value;
                }
                Self::SERVER_AGE => {
                    instance.server_age = value as u8;
                }
                _ => {}
            };
        }
        Some(instance)
    }
}

#[async_trait]
impl GSHandle for GSStatusUpdate {
    async fn handle(
        &self,
        gs: &mut GSHandler,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRun> {
        let lc = gs.get_lc();
        let mut updated = false;
        if let Some(server_id) = gs.server_id {
            updated = lc.with_gs(server_id, |gsi| {
                gsi.set_max_players(self.max_players);
                gsi.set_age_limit(self.server_age);
                gsi.use_square_brackets(self.use_square_brackets);
                gsi.set_server_type(self.server_type);
                gsi.set_server_status(self.status.clone() as i32);
            });
        }
        if !updated {
            return Err(PacketRun {
                msg: Some(format!("Server was not found, GS id {:?}", gs.server_id)),
                response: Some(Box::new(PlayerLogin::new(
                    PlayerLoginFailReasons::ReasonAccessFailed,
                ))),
            });
        }
        gs.start_channel();
        Ok(None)
    }
}
