use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum SystemMessageType {
    WelcomeToTheWorldOfLineage2 = 34, //Welcome to the World of Lineage II.
}

impl From<SystemMessageType> for u16 {
    fn from(value: SystemMessageType) -> Self {
        value as u16
    }
}
#[derive(Debug, Clone, SendablePacket)]
pub struct SystemMessage {
    pub buffer: SendablePacketBuffer,
}

impl SystemMessage {
    pub const PACKET_ID: u8 = 0x62;
    pub fn new(msg: SystemMessageType) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_u16(msg)?;
        inst.buffer.write(0)?; //todo: parameters
        Ok(inst)
    }
}

#[cfg(test)]
mod test {
    use crate::packets::to_client::system_message::{SystemMessage, SystemMessageType};

    #[tokio::test]
    async fn test_system_message() {
        let mut packet =
            SystemMessage::new(SystemMessageType::WelcomeToTheWorldOfLineage2).unwrap();
        assert_eq!([98, 34, 0, 0], packet.buffer.get_data_mut(false)[2..]);
    }
}
