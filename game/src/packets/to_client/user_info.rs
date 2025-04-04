use crate::controller::Controller;
use crate::data::classes::mapping::Class;
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
    pub async fn new(
        char_info: &CharacterInfo,
        user_info_flags: BitMask,
        controller: &Controller,
    ) -> anyhow::Result<Self> {
        //todo: check is subclass locked
        let mut block_size = 5;
        block_size += UserInfoType::calculate_block_size(&user_info_flags);
        let mut visible_name_size = 0;
        let exp_table = &controller.exp_table;
        if user_info_flags.contains_mask(UserInfoType::BasicInfo) {
            visible_name_size = u16::try_from(char_info.get_visible_name().len())? * 2;
            block_size += u32::from(visible_name_size);
        }
        let mut title_size = 0;
        if user_info_flags.contains_mask(UserInfoType::Clan) {
            title_size = u16::try_from(
                char_info
                    .char_model
                    .title
                    .as_ref()
                    .unwrap_or(&String::new())
                    .len(),
            )? * 2;
            block_size += title_size as u32;
        }
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            block_size,
            mask: user_info_flags,
            title: String::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(char_info.char_model.id)?;
        inst.buffer.write_u32(inst.block_size)?;
        inst.buffer.write_u16(23u16)?;
        inst.buffer.write_bytes(inst.mask.bytes())?;
        let is_cl = controller.clan_ally_manager.read().await.is_clan_leader(
            char_info.char_model.id,
            char_info.char_model.clan_id.unwrap(),
        );
        if inst.mask.contains_mask(UserInfoType::Relation) {
            inst.buffer.write_u32(char_info.get_relation(is_cl))?;
        }
        if inst.mask.contains_mask(UserInfoType::BasicInfo) {
            inst.buffer.write_u16(visible_name_size)?;
            inst.buffer
                .write_sized_c_utf16le_string(Some(char_info.get_visible_name()))?;
            inst.buffer.write_bool(char_info.is_gm())?;
            inst.buffer.write_i8(char_info.char_model.race_id)?;
            inst.buffer.write_bool(char_info.char_model.is_female)?;
            #[allow(clippy::cast_sign_loss)]
            let class_id = Class::try_from(char_info.char_model.class_id as u8)?;
            let template = controller.class_templates.try_get_template(class_id)?;
            inst.buffer.write_u32(class_id.get_root().id)?;
            inst.buffer
                .write_i32(i32::from(char_info.char_model.class_id))?;
            inst.buffer.write_i8(char_info.char_model.level)?;
        }
        if inst.mask.contains_mask(UserInfoType::BaseStats) {
            inst.buffer.write_u16(18u16)?;
            inst.buffer.write_u16(char_info.get_str());
            inst.buffer.write_u16(char_info.get_dex());
            inst.buffer.write_u16(char_info.get_con());
            inst.buffer.write_u16(char_info.get_int());
            inst.buffer.write_u16(char_info.get_wit());
            inst.buffer.write_u16(char_info.get_men());
            inst.buffer.write_u16(0u16);
            inst.buffer.write_u16(0u16);
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        if inst.mask.contains_mask(UserInfoType::MaxHpCpMp) {
            inst.buffer.write_u16(14u16)?;
            inst.buffer.write_u32(char_info.get_max_hp().round() as u32);
            inst.buffer.write_u32(char_info.get_max_mp().round() as u32);
            inst.buffer.write_u32(char_info.get_max_cp().round() as u32);
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        if inst.mask.contains_mask(UserInfoType::CurrentHpMpCpExpSp) {
            inst.buffer.write_u16(38u16)?;
            inst.buffer
                .write_u32(char_info.char_model.cur_hp.round() as u32);
            inst.buffer
                .write_u32(char_info.char_model.cur_mp.round() as u32);
            inst.buffer
                .write_u32(char_info.char_model.cur_cp.round() as u32);
            inst.buffer.write_i64(char_info.char_model.sp);
            inst.buffer.write_i64(char_info.char_model.exp);
            let current_threshold = exp_table.get_exp(char_info.char_model.level as u8);
            let next_threshold = exp_table.get_exp(char_info.char_model.level as u8);
            #[allow(clippy::cast_precision_loss)]
            inst.buffer.write_f64(
                (char_info.char_model.exp as f64 - current_threshold as f64)
                    / (next_threshold as f64 - current_threshold as f64),
            );
        }
        if inst.mask.contains_mask(UserInfoType::EnchantLevel) {
            inst.buffer.write_u16(4u16);
            inst.buffer.write(char_info.get_weapon_enchant());
            inst.buffer.write(char_info.get_min_armor_enchant());
        }
        if inst.mask.contains_mask(UserInfoType::Appearance) {
            inst.buffer.write_u16(15u16);
            
        }
        //todo: complete
        Ok(inst)
    }
}
