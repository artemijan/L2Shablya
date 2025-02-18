use crate::packets::enums::CharNameResponseVariant;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacketImpl)]
pub struct CreateCharFailed {
    buffer: SendablePacketBuffer,
    error: i32,
}

#[allow(unused)]
impl CreateCharFailed {
    const PACKET_ID: u8 = 0x10;
    const EX_PACKET_ID: Option<u16> = None;
    pub fn new(error: CharNameResponseVariant) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            error: error as i32,
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(inst.error)?;
        Ok(inst)
    }
}
#[cfg(test)]
mod tests {
    use crate::packets::enums::CharNameResponseVariant;
    use crate::packets::to_client::CreateCharFailed;

    #[test]
    fn test_char_create_fail() {
        let mut packet = CreateCharFailed::new(CharNameResponseVariant::InvalidName).unwrap();
        assert_eq!([7, 0, 16, 4, 0, 0, 0], packet.buffer.get_data_mut(false));
    }
}
