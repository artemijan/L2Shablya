use l2_core::shared_packets::write::SendablePacketBuffer;
use std::fmt::Debug;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemMessageType {
    WelcomeToTheWorldOfLineage2 = 34, //Welcome to the World of Lineage II.
    YouHitForS1Damage = 35,          // You hit for $s1 damage.
    C1HitYouForS2Damage = 36,        // $c1 hit you for $s2 damage.
    C1SAttackWentAstray = 2265,      // $c1's attack went astray.
    C1HasInflictedS3DamageOnC2 = 2261, // $c1 has inflicted $s3 damage on $c2.
    C1HasReceivedS3DamageFromC2 = 2262, // $c1 has received $s3 damage from $c2.
    C1HasReceivedS3DamageThroughC2 = 2263, // $c1 has received $s3 damage through $c2.
    S1_2 = 1983,                     // $s1
    S1IsNotAvailableAtThisTimeBeingPreparedForReuse = 48, // $s1 is not available at this time: being prepared for reuse.
}

impl From<SystemMessageType> for u16 {
    fn from(value: SystemMessageType) -> Self {
        value as u16
    }
}

pub enum SystemMessageParam {
    Text(String),
    Int(i32),
    Long(i64),
    ItemName(i32),
    NpcName(i32),
    PcName(String),
    SkillName { id: i32, level: i16, sub_level: i16 },
    Popup { target: i32, attacker: i32, damage: i32 },
    ZoneName { x: i32, y: i32, z: i32 },
    CastleId(i32),
    ElementId(i32),
    InstanceName(i32),
    DoorName(i32),
    SystemString(i32),
    ClassId(i32),
    Byte(i32),
    FactionId(i32),
}

impl SystemMessageParam {
    pub fn get_type(&self) -> u8 {
        match self {
            Self::Text(_) => 0,
            Self::Int(_) => 1,
            Self::NpcName(_) => 2,
            Self::ItemName(_) => 3,
            Self::SkillName { .. } => 4,
            Self::CastleId(_) => 5,
            Self::Long(_) => 6,
            Self::ZoneName { .. } => 7,
            Self::ElementId(_) => 9,
            Self::InstanceName(_) => 10,
            Self::DoorName(_) => 11,
            Self::PcName(_) => 12,
            Self::SystemString(_) => 13,
            Self::ClassId(_) => 15,
            Self::Popup { .. } => 16,
            Self::Byte(_) => 20,
            Self::FactionId(_) => 24,
        }
    }

    pub fn write_to(&self, buffer: &mut SendablePacketBuffer) -> anyhow::Result<()> {
        buffer.write_u8(self.get_type())?;
        match self {
            Self::Text(s) | Self::PcName(s) => buffer.write_c_utf16le_string(Some(s.as_str()))?,
            Self::Int(i) | Self::NpcName(i) | Self::ItemName(i) | Self::DoorName(i) => {
                buffer.write_i32(*i)?
            }
            Self::Long(l) => buffer.write_i64(*l)?,
            Self::SkillName {
                id,
                level,
                sub_level,
            } => {
                buffer.write_i32(*id)?;
                buffer.write_i16(*level)?;
                buffer.write_i16(*sub_level)?;
            }
            Self::Popup {
                target,
                attacker,
                damage,
            }
            | Self::ZoneName {
                x: target,
                y: attacker,
                z: damage,
            } => {
                buffer.write_i32(*target)?;
                buffer.write_i32(*attacker)?;
                buffer.write_i32(*damage)?;
            }
            Self::CastleId(i) | Self::SystemString(i) | Self::InstanceName(i) | Self::ClassId(i) => {
                buffer.write_i16(*i as i16)?
            }
            Self::Byte(i) | Self::ElementId(i) | Self::FactionId(i) => buffer.write_u8(*i as u8)?,
        }
        Ok(())
    }
}

pub struct SystemMessage {
    pub message: SystemMessageType,
    pub params: Vec<SystemMessageParam>,
}

impl SystemMessage {
    pub const PACKET_ID: u8 = 0x62;
    pub fn new(message: SystemMessageType) -> anyhow::Result<Self> {
        Ok(Self {
            message,
            params: Vec::new(),
        })
    }

    pub fn add_param(&mut self, param: SystemMessageParam) -> anyhow::Result<()> {
        self.params.push(param);
        Ok(())
    }
}

impl l2_core::shared_packets::common::SendablePacket for SystemMessage {
    fn get_buffer(self) -> SendablePacketBuffer {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID).unwrap();
        buffer.write_u16(self.message).unwrap();
        buffer.write_u8(self.params.len() as u8).unwrap();
        for param in self.params {
            param.write_to(&mut buffer).unwrap();
        }
        buffer
    }

    fn name(&self) -> &'static str {
        "SystemMessage"
    }
}

#[cfg(test)]
mod test {
    use l2_core::shared_packets::common::SendablePacket;
    use crate::packets::to_client::system_message::{SystemMessage, SystemMessageType};

    #[tokio::test]
    async fn test_system_message() {
        let packet =
            SystemMessage::new(SystemMessageType::WelcomeToTheWorldOfLineage2).unwrap();
        let mut buffer = packet.get_buffer();
        // [Size(2), ID(1), MsgID(2), Count(1)]
        assert_eq!([6, 0, 98, 34, 0, 0], buffer.get_data_mut(false));
    }

    #[tokio::test]
    async fn test_system_message_with_params() {
        let mut packet = SystemMessage::new(SystemMessageType::YouHitForS1Damage).unwrap();
        packet.add_param(super::SystemMessageParam::Int(100)).unwrap();
        
        let mut buffer = packet.get_buffer();
        let data = buffer.get_data_mut(false);
        // Size(2), ID(1), MsgID(2), Count(1), ParamType(1), ParamValue(4)
        assert_eq!(data[2], 0x62); // ID
        assert_eq!(data[5], 1);    // Count
        assert_eq!(data[6], 1);    // ParamType (Int)
        assert_eq!(data[7], 100);  // ParamValue
    }
}
