use crate as l2_core;
use crate::config::gs::GSServer;
use crate::shared_packets::common::{GSStatus, ReadablePacket};
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[derive(Clone, Debug, Default, SendablePacketImpl)]
pub struct GSStatusUpdate {
    buffer: SendablePacketBuffer,
    pub status: GSStatus,
    pub use_square_brackets: bool,
    pub max_players: u32,
    pub server_type: i32,
    pub server_age: u8,
}

#[allow(unused)]
impl GSStatusUpdate {
    const SERVER_LIST_STATUS: i32 = 0x01;
    const SERVER_TYPE: i32 = 0x02;
    const SERVER_LIST_SQUARE_BRACKET: i32 = 0x03;
    const MAX_PLAYERS: i32 = 0x04;
    const TEST_SERVER: i32 = 0x05;
    const SERVER_AGE: i32 = 0x06;

    #[allow(clippy::missing_errors_doc)]
    pub fn new(cfg: &GSServer) -> anyhow::Result<Self> {
        let mut inst = Self {
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
        inst.write_all()?;
        Ok(inst)
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(0x06)?;
        let fields = [
            (Self::SERVER_LIST_STATUS, self.status as i32),
            (Self::SERVER_TYPE, self.server_type),
            (
                Self::SERVER_LIST_SQUARE_BRACKET,
                i32::from(self.use_square_brackets),
            ),
            (Self::MAX_PLAYERS, self.max_players as i32),
            (Self::SERVER_AGE, i32::from(self.server_age)),
        ];
        self.buffer.write_u32(fields.len() as u32)?;
        for (f, v) in fields {
            self.buffer.write_i32(f)?;
            self.buffer.write_i32(v)?;
        }
        Ok(())
    }
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
impl ReadablePacket for GSStatusUpdate {
    const PACKET_ID: u8 = 0x06;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?; //packet id
        let size = buffer.read_i32()? as usize;
        let mut instance = Self::default();
        for _ in 0..size {
            let gs_type = buffer.read_i32()?;
            let value = buffer.read_i32()?;

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
        Ok(instance)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::traits::ServerConfig;

    #[test]
    fn gs_status_update_new() {
        let cfg = GSServer::load("../config/game.yaml");
        let mut pack = GSStatusUpdate::new(&cfg).unwrap();
        assert!(!pack.use_square_brackets);
        assert_eq!(pack.max_players, 5000);
        assert_eq!(pack.server_type, 1);
        assert_eq!(pack.server_age, 12);
        assert_eq!(pack.status, GSStatus::Auto);
        assert_eq!(
            pack.buffer.get_data_mut(false),
            [
                47, 0, 6, 5, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0,
                0, 0, 0, 0, 4, 0, 0, 0, 136, 19, 0, 0, 6, 0, 0, 0, 12, 0, 0, 0
            ]
        );
    }
    #[test]
    fn gs_status_update_read() {
        let bytes = [
            6, 5, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0,
            4, 0, 0, 0, 136, 19, 0, 0, 6, 0, 0, 0, 12, 0, 0, 0,
        ];
        let pack = GSStatusUpdate::read(&bytes).unwrap();
        assert!(!pack.use_square_brackets);
        assert_eq!(pack.max_players, 5000);
        assert_eq!(pack.server_type, 1);
        assert_eq!(pack.server_age, 12);
        assert_eq!(pack.status, GSStatus::Auto);
    }
}
