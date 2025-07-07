use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct PledgeWaitingListAlarm {
    pub(crate) buffer: SendablePacketBuffer,
}

impl PledgeWaitingListAlarm {
    const PACKET_ID: u8 = 0xFE;
    const EX_PACKET_ID: u16 = 0x147;
    pub fn new() -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(Self::EX_PACKET_ID)?;
        Ok(inst)
    }
}

#[cfg(test)]
mod test {
    use l2_core::shared_packets::common::SendablePacket;

    use crate::packets::to_client::extended::PledgeWaitingListAlarm;

    #[tokio::test]
    async fn test_henna_ok() {
        let p = PledgeWaitingListAlarm::new().unwrap();
        assert_eq!(
            [
                254, 71, 1
            ],
            p.get_buffer().get_data_mut(false)[2..]
        );
    }
}
