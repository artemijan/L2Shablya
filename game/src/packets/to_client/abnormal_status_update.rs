use l2_core::game_objects::creature::buff::AppliedBuff;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct AbnormalStatusUpdate {
    pub(crate) buffer: SendablePacketBuffer,
}

impl AbnormalStatusUpdate {
    pub const PACKET_ID: u8 = 0x85;

    pub fn new(buffs: &[AppliedBuff]) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(u16::try_from(buffs.len())?)?;
        for buff in buffs {
            inst.buffer.write_i32(buff.skill_id)?;
            inst.buffer
                .write_u16(u16::try_from(buff.skill_level.max(1))?)?;
            inst.buffer.write_u16(0u16)?; // sub level
            inst.buffer.write_i32(0)?; // abnormal type client id
            // optional int: seconds left, -1 for permanent effects
            let time = buff.remaining_secs();
            if time >= i32::from(i16::MAX) {
                inst.buffer.write_i16(i16::MAX)?;
                inst.buffer.write_i32(time)?;
            } else {
                inst.buffer.write_i16(i16::try_from(time)?)?;
            }
        }
        Ok(inst)
    }
}
#[cfg(test)]
mod test {
    use crate::packets::to_client::AbnormalStatusUpdate;

    #[tokio::test]
    async fn test_empty_abnormal_status() {
        let mut packet = AbnormalStatusUpdate::new(&[]).unwrap();
        assert_eq!([133, 0, 0], packet.buffer.get_data_mut(false)[2..]);
    }

    #[tokio::test]
    async fn test_with_buff() {
        use chrono::{Duration, Utc};
        use l2_core::game_objects::creature::buff::AppliedBuff;
        let buff = AppliedBuff {
            skill_id: 1068,
            skill_level: 1,
            caster_id: 1,
            abnormal_type: Some("PA_UP".to_string()),
            abnormal_level: 1,
            end_time: Utc::now() + Duration::seconds(100),
            mods: vec![],
        };
        let mut packet = AbnormalStatusUpdate::new(&[buff]).unwrap();
        let data = packet.buffer.get_data_mut(false).to_vec();
        assert_eq!(data[2], 133); // packet id
        assert_eq!(data[3..5], [1, 0]); // count
        assert_eq!(data[5..9], 1068i32.to_le_bytes()); // skill id
    }
}
