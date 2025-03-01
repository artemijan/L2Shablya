use entities::dao::char_info::CharacterInfo;
use l2_core::bitmask::BitMask;
use l2_core::model::user_info::UserInfoType;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacketImpl)]
pub struct UserInfo {
    buffer: SendablePacketBuffer,
    block_size: u32,
    mask: BitMask,
    title: String,
}

#[allow(unused)]
impl UserInfo {
    const PACKET_ID: u8 = 0x32;
    const EX_PACKET_ID: Option<u16> = None;
    pub fn new(char_info: &CharacterInfo, user_info_flags: BitMask) -> anyhow::Result<Self> {
        //todo: check is subclass locked
        let mut block_size = 5;
        block_size += UserInfoType::calculate_block_size(&user_info_flags);
        if user_info_flags.contains_mask(UserInfoType::BasicInfo as u32) {
            block_size += u32::try_from(char_info.char_model.name.len())? * 2;
        }
        if user_info_flags.contains_mask(UserInfoType::Clan as u32) {
            block_size += u32::try_from(char_info
                .char_model
                .title
                .as_ref()
                .unwrap_or(&String::new())
                .len())? * 2;
        }
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            block_size,
            mask: user_info_flags,
            title: String::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(char_info.char_model.id)?;
        inst.buffer.write_i32(char_info.char_model.id)?;
        //todo: complete 
        Ok(inst)
    }
}
