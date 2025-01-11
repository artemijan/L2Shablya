use crate::controller::Controller;
use l2_core::packets::common::SendablePacket;
use l2_core::packets::write::SendablePacketBuffer;

#[derive(Debug, Clone)]
pub struct NewCharacterResponse {
    buffer: SendablePacketBuffer,
}

impl NewCharacterResponse {
    const PACKET_ID: i32 = 0x0D;
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    pub fn new(controller: &Controller) -> anyhow::Result<Self> {
        let templates = controller
            .class_templates
            .get_available_templates_for_registration();
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write_i32(Self::PACKET_ID)?;
        inst.buffer.write_i32(templates.len() as i32)?;
        for t in &templates {
            inst.buffer.write_i32(t.class_id.get_class().race as i32)?;
            inst.buffer.write_i32(t.class_id as i32)?;
            inst.buffer.write_i32(99)?;
            inst.buffer.write_i32(t.static_data.base_str)?;
            inst.buffer.write_i32(1)?;
            inst.buffer.write_i32(99)?;
            inst.buffer.write_i32(t.static_data.base_dex)?;
            inst.buffer.write_i32(1)?;
            inst.buffer.write_i32(99)?;
            inst.buffer.write_i32(t.static_data.base_con)?;
            inst.buffer.write_i32(1)?;
            inst.buffer.write_i32(99)?;
            inst.buffer.write_i32(t.static_data.base_int)?;
            inst.buffer.write_i32(1)?;
            inst.buffer.write_i32(99)?;
            inst.buffer.write_i32(t.static_data.base_wit)?;
            inst.buffer.write_i32(1)?;
            inst.buffer.write_i32(99)?;
            inst.buffer.write_i32(t.static_data.base_men)?;
            inst.buffer.write_i32(1)?;
        }
        Ok(inst)
    }
}
impl SendablePacket for NewCharacterResponse {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
