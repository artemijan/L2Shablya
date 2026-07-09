use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacket;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum StatusUpdateType {
    Level = 0x01,
    Exp = 0x02,
    Str = 0x03,
    Dex = 0x04,
    Con = 0x05,
    Int = 0x06,
    Wit = 0x07,
    Men = 0x08,
    CurHp = 0x09,
    MaxHp = 0x0a,
    CurMp = 0x0b,
    MaxMp = 0x0c,
    CurLoad = 0x0e,
    PAtk = 0x11,
    AtkSpd = 0x12,
    PDef = 0x13,
    Evasion = 0x14,
    Accuracy = 0x15,
    Critical = 0x16,
    MAtk = 0x17,
    CastSpd = 0x18,
    MDef = 0x19,
    PvpFlag = 0x1a,
    Reputation = 0x1b,
    CurCp = 0x21,
    MaxCp = 0x22,
}

#[derive(Clone, Debug, SendablePacket)]
pub struct StatusUpdate {
    pub buffer: SendablePacketBuffer,
}

impl StatusUpdate {
    pub const PACKET_ID: u8 = 0x18;

    pub fn new(object_id: i32) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(object_id)?;
        inst.buffer.write_i32(0)?; // visible caster
        inst.buffer.write_u8(0)?; // is visible
        inst.buffer.write_u8(0)?; // attributes count (to be updated later)

        Ok(inst)
    }

    pub fn add_update(&mut self, update_type: StatusUpdateType, value: i32) -> anyhow::Result<()> {
        self.buffer.write_u8(update_type as u8)?;
        self.buffer.write_i32(value)?;

        // Update attributes count byte
        let data = self.buffer.get_data_mut(false);
        data[11] += 1;

        Ok(())
    }
}
