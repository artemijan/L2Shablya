use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct ShortcutsInit {
    buffer: SendablePacketBuffer,
}

impl ShortcutsInit {
    const PACKET_ID: u8 = 0x45;

    pub fn new(p: &Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(0u16)?; //count
        //todo: implement me
        Ok(inst)
    }
}
