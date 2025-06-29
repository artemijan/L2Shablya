use l2_core::game_objects::player::Player;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct CharEtcStatusUpdate {
    pub buffer: SendablePacketBuffer,
}

#[allow(unused)]
impl CharEtcStatusUpdate {
    const PACKET_ID: u8 = 0xF9;
    const EX_PACKET_ID: Option<u16> = None;
    pub fn new(p:&Player) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write(p.get_charges())?;// 1-7 increase force, level
        inst.buffer.write_i32(p.get_weight_penalty())?;
        inst.buffer.write(p.get_expertise_weapon_penalty())?;
        inst.buffer.write(p.get_expertise_armor_penalty())?;
        inst.buffer.write(0); // Death Penalty [1-15, 0 = disabled)], not used anymore in Ertheia
        inst.buffer.write(p.get_charged_souls())?;
        let mut mask:i32 = (p.block_all() || p.chat_banned() || p.silence_mode()).into();
        mask |= if p.is_in_instance_zone() { 2 } else { 0 };
        mask |= if p.has_charm_of_courage() { 4 } else { 0 };
        inst.buffer.write(u8::try_from(mask)?)?;
        Ok(inst)
    }
}
