use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct CreateCharOk {
    pub buffer: SendablePacketBuffer,
}

#[allow(unused)]
impl CreateCharOk {
    const PACKET_ID: u8 = 0x0F;
    const EX_PACKET_ID: Option<u16> = None;
    pub fn new() -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(1)?;
        Ok(inst)
    }
}

#[cfg(test)]
mod tests {
    use crate::packets::to_client::CreateCharOk;

    #[test]
    fn test_create_ok() {
        let mut created = CreateCharOk::new().unwrap();
        assert_eq!([7, 0, 15, 1, 0, 0, 0], created.buffer.get_data_mut(false));
    }
}
