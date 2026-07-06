use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[derive(Clone, Debug, SendablePacket)]
pub struct ActionFailed {
    pub buffer: SendablePacketBuffer,
}

#[repr(i32)]
#[derive(Debug)]
enum ActionFailReasons {
    Simultaneous = -1,
    Normal = 0,
    NormalSecond = 1,
    Blue = 2,
    Green = 3,
    Red = 4,
}
impl ActionFailed {
    pub const PACKET_ID: u8 = 0x1F;
    pub fn simultaneous() -> anyhow::Result<Self> {
        Self::new(ActionFailReasons::Simultaneous)
    }
    pub fn normal() -> anyhow::Result<Self> {
        Self::new(ActionFailReasons::Normal)
    }
    pub fn normal_second() -> anyhow::Result<Self> {
        Self::new(ActionFailReasons::NormalSecond)
    }
    pub fn blue() -> anyhow::Result<Self> {
        Self::new(ActionFailReasons::Blue)
    }
    pub fn green() -> anyhow::Result<Self> {
        Self::new(ActionFailReasons::Green)
    }
    pub fn red() -> anyhow::Result<Self> {
        Self::new(ActionFailReasons::Red)
    }
    fn new(reason: ActionFailReasons) -> anyhow::Result<Self> {
        println!("Writing reason: {reason:?}");
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(reason as i32)?;
        Ok(inst)
    }
}
