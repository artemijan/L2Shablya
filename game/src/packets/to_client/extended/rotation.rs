use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Rotation {
    pub(crate) buffer: SendablePacketBuffer,
}

impl Rotation {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0xC2;

    pub fn new(player: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer.write_i32(player.char_model.id)?;
        inst.buffer.write_i32(player.location.heading)?;
        Ok(inst)
    }
}
