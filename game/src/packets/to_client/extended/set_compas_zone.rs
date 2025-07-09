use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, Default, SendablePacket)]
pub struct SetCompasZoneCode {
    pub buffer: SendablePacketBuffer,
}

impl SetCompasZoneCode {
    pub const PACKET_ID: u8 = 0xFE;
    pub const EX_PACKET_ID: u16 = 0x33;
    pub fn new(compas_zone: i32) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_u16(Self::EX_PACKET_ID)?;
        buffer.write_i32(compas_zone)?;
        Ok(Self { buffer })
    }
}
#[cfg(test)]
mod test {
    use crate::packets::to_client::extended::SetCompasZoneCode;
    use l2_core::shared_packets::common::SendablePacket;

    #[tokio::test]
    async fn test_compas_zone_code() {
        let p = SetCompasZoneCode::new(0x0C).unwrap();
        assert_eq!(
            [254, 51, 0, 12, 0, 0, 0],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}
