use l2_core::config::gs::GSServer;
use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacketImpl)]
pub struct VitalityInfo {
    buffer: SendablePacketBuffer,
    points: i32,
    vitality_bonus: i32,
    vitality_items_remaining: i32,
}

impl VitalityInfo {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x118;

    pub fn new(player: &Player, config: &GSServer) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            points: player.char_model.vitality_points,
            vitality_bonus: player.get_vitality_bonus(),
            vitality_items_remaining: config.vitality_max_items_allowed
                - player.get_vitality_used(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_i32(inst.points)?;
        inst.buffer.write_i32(inst.vitality_bonus)?;
        inst.buffer.write_u16(0u16)?; // Vitality additional bonus in %
        inst.buffer
            .write_u16(inst.vitality_items_remaining as u16)?;
        inst.buffer
            .write_u16(config.vitality_max_items_allowed as u16)?;
        Ok(inst)
    }
}
