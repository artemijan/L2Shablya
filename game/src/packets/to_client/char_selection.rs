use crate::controller::Controller;
use entities::entities::item;
use l2_core::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;
use std::sync::Arc;
use l2_core::game_objects::player::paper_doll::PaperDoll;
use l2_core::game_objects::player::Player;

#[allow(unused)]
#[derive(Debug, Clone, Default, SendablePacketImpl)]
pub struct CharSelectionInfo {
    buffer: SendablePacketBuffer,
    session_id: i32,
    active_id: i32,
}

impl CharSelectionInfo {
    pub const PACKET_ID: u8 = 0x09;
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::too_many_lines)]
    pub fn new(
        account_name: &str,
        session_id: i32,
        controller: &Arc<Controller>,
        chars: &[Player],
    ) -> anyhow::Result<Self> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        let char_len = chars.len() as u32;
        buffer.write_u32(char_len)?;
        let cfg = controller.get_cfg();
        let exp_table = &controller.exp_table;
        buffer.write_u32(u32::from(cfg.max_chars_on_account))?;
        buffer.write_bool(char_len == u32::from(cfg.max_chars_on_account))?;
        buffer.write(1)?; // 0=can't play, 1=can play free until level 85, 2=100% free play
        buffer.write_u32(2u32)?; // if 1, Korean client
        buffer.write(0)?; // Balthus Knights, if 1 suggests premium account
        let mut last_access = None;
        let mut active_id = -1;
        for (index, char_info) in chars.iter().enumerate() {
            let char = &char_info.char_model;
            if char.last_access > last_access && char.delete_at.is_none() {
                last_access = char.last_access;
                active_id = index as i32;
            }
            buffer.write_c_utf16le_string(Some(&char.name))?;
            buffer.write_i32(char.id)?;
            buffer.write_c_utf16le_string(Some(account_name))?;
            buffer.write_i32(session_id)?;
            buffer.write_i32(0)?; // clan id
            buffer.write_i32(0)?; // Builder level
            buffer.write_i32(i32::from(char.is_female))?;
            buffer.write_i32(i32::from(char.race_id))?;
            buffer.write_i32(i32::from(char.base_class_id))?;
            buffer.write_i32(1)?; // GameServerName
            buffer.write_i32(char.x)?;
            buffer.write_i32(char.y)?;
            buffer.write_i32(char.z)?;
            buffer.write_f64(char.cur_hp)?;
            buffer.write_f64(char.cur_mp)?;
            buffer.write_i64(char.sp)?;
            buffer.write_i64(char.exp)?;
            let exp_current = exp_table.get_exp(char.get_lvl());
            let exp_next = exp_table.get_exp_for_next_lvl(char.get_lvl());
            #[allow(clippy::cast_precision_loss)]
            buffer.write_f64((char.exp - exp_current) as f64 / (exp_next - exp_current) as f64)?;
            buffer.write_i32(i32::from(char.level))?;
            buffer.write_i32(char.reputation)?;
            buffer.write_i32(char.pk_kills)?;
            buffer.write_i32(char.pvp_kills)?;
            buffer.write_i32(0)?;
            buffer.write_i32(0)?;
            buffer.write_i32(0)?;
            buffer.write_i32(0)?;
            buffer.write_i32(0)?;
            buffer.write_i32(0)?;
            buffer.write_i32(0)?;
            buffer.write_i32(0)?; // Ertheia
            buffer.write_i32(0)?; // Ertheia
            for slot in PaperDoll::ordered_ids() {
                buffer.write_i32(char_info.try_get_paper_doll_item_id(slot)?)?;
            }
            for v_slot in PaperDoll::visual_ids() {
                buffer.write_i32(char_info.try_get_paper_doll_visual_id(v_slot)?)?;
            }
            buffer.write_i16(char_info.get_enchant_effect(PaperDoll::Chest))?; // Upper Body enchant level
            buffer.write_i16(char_info.get_enchant_effect(PaperDoll::Legs))?; // Lower Body enchant level
            buffer.write_i16(char_info.get_enchant_effect(PaperDoll::Head))?; // Headgear enchant level
            buffer.write_i16(char_info.get_enchant_effect(PaperDoll::Gloves))?; // Gloves enchant level
            buffer.write_i16(char_info.get_enchant_effect(PaperDoll::Feet))?; // Boots enchant level
            buffer.write_i32(char_info.get_hair_style())?;
            buffer.write_i32(char_info.get_hair_color())?;
            buffer.write_i32(char_info.get_face())?;
            buffer.write_f64(char_info.get_max_hp())?; // Maximum HP
            buffer.write_f64(char_info.get_max_mp())?; // Maximum MP
            buffer.write_i32(char_info.get_delete_timer())?;
            buffer.write_i32(i32::from(char_info.char_model.class_id))?; // Class ID
            #[allow(clippy::cast_lossless)]
            buffer.write_i32((index as i32 == active_id) as i32)?; // is_active
            buffer.write(char_info.get_enchant_effect_as_byte(PaperDoll::RHand))?;
            let aug = char_info
                .get_weapon()
                .and_then(item::Model::get_augmentation);
            if let Some(augmentation) = aug {
                buffer.write_i32(augmentation.1)?;
                buffer.write_i32(augmentation.2)?;
            } else {
                buffer.write_i32(0)?;
                buffer.write_i32(0)?;
            };
            buffer.write_i32(char_info.get_transform_id())?;
            buffer.write_i32(0)?; // Pet NpcId
            buffer.write_i32(0)?; // Pet level
            buffer.write_i32(0)?; // Pet food
            buffer.write_i32(0)?; // Pet food level
            buffer.write_f64(0.0)?; // Current pet HP
            buffer.write_f64(0.0)?; // Current pet MP
            buffer.write_i32(char_info.char_model.vitality_points)?;
            buffer.write_i32(cfg.rates.vitality_exp_multiplier * 100)?;
            buffer.write_i32(char_info.get_vitality_used())?;
            buffer.write_i32(i32::from(char_info.char_model.access_level != -100))?;
            buffer.write_bool(char_info.char_model.nobless)?;
            buffer.write(
                if controller.hero_list.contains_key(&char_info.char_model.id) {
                    2
                } else {
                    0
                },
            )?;
            buffer.write_bool(char_info.is_hair_accessory_enabled())?;
        }
        Ok(Self {
            buffer,
            session_id,
            active_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_gs_config;
    use entities::entities::character;

    #[tokio::test]
    async fn test_char_selected() {
        let inst = character::Model {
            name: "test".to_string(),
            level: 1,
            face: 1,
            hair_style: 2,
            hair_color: 2,
            is_female: false,
            delete_at: None,
            user_id: 1,
            ..Default::default()
        };
        let char = Player::new(inst, vec![]).unwrap();
        let cfg = get_gs_config();
        let controller = Arc::new(Controller::from_config(Arc::new(cfg)));
        let mut packet = CharSelectionInfo::new("admin", 1, &controller, &vec![char]).unwrap();
        assert_eq!(
            [
                202, 1, 9, 1, 0, 0, 0, 7, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 116, 0, 101, 0, 115, 0,
                116, 0, 0, 0, 0, 0, 0, 0, 97, 0, 100, 0, 109, 0, 105, 0, 110, 0, 0, 0, 1, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1
            ],
            packet.buffer.get_data_mut(false)
        );
    }
}
