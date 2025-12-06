use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct TargetSelected {
    pub buffer: SendablePacketBuffer,
}

impl TargetSelected {
    pub const PACKET_ID: u8 = 0xB9;

    pub fn new(target_id: i32, level_diff: i16) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(1)?; // Grand Crusade
        inst.buffer.write_i32(target_id)?;
        inst.buffer.write_i16(level_diff)?;
        inst.buffer.write_i32(0)?;
        Ok(inst)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_target_selected() {
        let mut packet = TargetSelected::new(1, 9).unwrap();
        assert_eq!(
            [185, 1, 0, 0, 0, 1, 0, 0, 0, 9, 0, 0, 0, 0, 0],
            packet.buffer.get_data_mut(false)[2..]
        );
    }
}
