use crate::controller::Controller;
use l2_core::shared_packets::write::SendablePacketBuffer;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ActionList {
    pub(crate) buffer: SendablePacketBuffer,
}

impl ActionList {
    const PACKET_ID: u8 = 0x11;
    const EX_PACKET_ID: u16 = 0x60;

    pub fn new(ctrl: &Controller) -> anyhow::Result<Self> {
        //todo: optimization, this is static packet we should cache it
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        inst.buffer
            .write_u32(u32::try_from(ctrl.action_list.actions.len())?)?;
        for al in &ctrl.action_list.actions {
            inst.buffer.write_u32(al.id)?;
        }
        Ok(inst)
    }
}
