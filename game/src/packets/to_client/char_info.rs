use l2_core::config::gs::GSServerConfig;
use l2_core::game_objects::player::effect::abnormal_effect::AbnormalVisualEffect;
use l2_core::game_objects::player::paper_doll::PaperDoll;
use l2_core::game_objects::player::Player;
use l2_core::game_objects::zone::ZoneId;
use l2_core::shared_packets::write::SendablePacketBuffer;
use l2_core::traits::conversion::ToU32Rounded;
use macro_common::SendablePacket;
use std::fmt::Debug;

#[derive(Debug, Clone, SendablePacket)]
pub struct CharInfo {
    pub(crate) buffer: SendablePacketBuffer,
}

impl CharInfo {
    const PACKET_ID: u8 = 0x31;
    const PAPERDOLL_SLOTS: [PaperDoll; 12] = [
        PaperDoll::Under,
        PaperDoll::Head,
        PaperDoll::RHand,
        PaperDoll::LHand,
        PaperDoll::Gloves,
        PaperDoll::Chest,
        PaperDoll::Legs,
        PaperDoll::Feet,
        PaperDoll::Cloak,
        PaperDoll::RHand,
        PaperDoll::Hair,
        PaperDoll::Hair2,
    ];
    const PAPERDOLL_AUGMENT_SLOTS: [PaperDoll; 3] =
        [PaperDoll::RHand, PaperDoll::LHand, PaperDoll::RHand];
    const PAPERDOLL_VISUAL_SLOTS: [PaperDoll; 9] = [
        PaperDoll::RHand,
        PaperDoll::LHand,
        PaperDoll::RHand,
        PaperDoll::Gloves,
        PaperDoll::Chest,
        PaperDoll::Legs,
        PaperDoll::Feet,
        PaperDoll::Hair,
        PaperDoll::Hair2,
    ];

    #[allow(clippy::too_many_lines)]
    pub fn new(p: &Player, cfg: &GSServerConfig) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write(0)?; // Grand Crusade
        inst.buffer.write_i32(p.get_x())?; // Confirmed
        inst.buffer.write_i32(p.get_y())?; // Confirmed
        inst.buffer.write_i32(p.get_z())?; // Confirmed
        inst.buffer
            .write_i32(p.get_vehicle_object_id().unwrap_or(0))?; // Confirmed
        inst.buffer.write_i32(p.get_object_id())?; // Confirmed
        inst.buffer
            .write_c_utf16le_string(Some(p.get_visible_name()))?; // Confirmed
        inst.buffer.write_i16(p.char_model.race_id)?; // Confirmed
        inst.buffer.write(p.char_model.is_female)?; // Confirmed
        inst.buffer.write_u32(p.template.class_id.get_root().id)?; // Confirmed
        for slot in &Self::PAPERDOLL_SLOTS {
            inst.buffer
                .write_i32(p.try_get_paper_doll_visual_id(*slot)?)?;
        }
        for slot in &Self::PAPERDOLL_AUGMENT_SLOTS {
            let aug = p
                .get_item(p.try_get_paper_doll_item_id(*slot)?)
                .and_then(|i| i.item_model.get_augmentation())
                .unwrap_or((0, 0, 0));
            inst.buffer.write_i32(aug.1)?;
            inst.buffer.write_i32(aug.2)?;
        }
        inst.buffer.write(p.get_min_armor_enchant())?;
        for slot in &Self::PAPERDOLL_VISUAL_SLOTS {
            inst.buffer
                .write_i32(p.try_get_paper_doll_visual_id(*slot)?)?;
        }
        inst.buffer.write(p.get_pvp_flag())?;
        inst.buffer.write_u32(p.char_model.reputation)?;
        inst.buffer.write_u32(p.get_m_atk_spd())?;
        inst.buffer.write_u32(p.get_p_atk_spd())?;
        inst.buffer.write_u16(p.get_run_speed())?;
        inst.buffer.write_u16(p.get_walk_speed())?;
        inst.buffer.write_u16(p.get_swim_run_speed())?;
        inst.buffer.write_u16(p.get_swim_walk_speed())?;
        inst.buffer.write_u16(p.get_fly_run_speed())?;
        inst.buffer.write_u16(p.get_fly_walk_speed())?;
        //todo: wtf is this? We need to write fly speed 2 times?
        inst.buffer.write_u16(p.get_fly_run_speed())?;
        inst.buffer.write_u16(p.get_fly_walk_speed())?;

        inst.buffer.write_f64(p.get_movement_speed_multiplier())?;
        inst.buffer.write_f64(p.get_p_atk_spd_multiplier())?;
        inst.buffer.write_f64(p.get_collision_radius())?;
        inst.buffer.write_f64(p.get_collision_height())?;
        inst.buffer.write_i32(p.get_hair_style())?;
        inst.buffer.write_i32(p.get_hair_color())?;
        inst.buffer.write_i32(p.get_face())?;
        inst.buffer
            .write_c_utf16le_string(p.char_model.title.as_deref())?; //todo: check if admin invisible
        inst.buffer.write_i32(p.get_clan_id())?;
        inst.buffer.write_i32(p.get_clan_crest_id())?;
        inst.buffer.write_i32(p.get_ally_id())?;
        inst.buffer.write_i32(p.get_ally_crest_id())?;
        inst.buffer.write(!p.is_sitting())?; // Confirmed
        inst.buffer.write(p.is_running())?; // Confirmed
        inst.buffer.write(p.is_in_combat())?; // Confirmed
        inst.buffer
            .write(!p.is_in_olympiad_mode() && p.is_alike_dead())?; // Confirmed
        inst.buffer.write(p.is_invincible())?;
        inst.buffer.write(p.get_mount_type())?; // 1-on Strider, 2-on Wyvern, 3-on Great Wolf, 0-no mount
        inst.buffer.write(p.get_private_store_type().id())?; // Confirmed

        inst.buffer
            .write_u16(u16::try_from(p.get_cubics().len())?)?; // Confirmed
        p.get_cubics()
            .iter()
            .try_for_each(|v| inst.buffer.write_u16(*v))?;

        inst.buffer.write(p.is_in_matching_room())?; // Confirmed
        inst.buffer.write(if p.is_inside_zone(ZoneId::Water) {
            1
        } else if p.is_flying_mounted() {
            2
        } else {
            0
        })?;
        inst.buffer.write_u16(p.get_recommendations_have())?; // Confirmed
        inst.buffer.write_i32(if p.get_mount_npc_id() == 0 {
            0
        } else {
            p.get_mount_npc_id() + 1_000_000
        })?;
        inst.buffer.write_u32(p.template.class_id)?; // Confirmed
        inst.buffer.write_i32(0)?; // TODO: Find me!
        inst.buffer.write(if p.is_mounted() {
            0
        } else {
            p.get_weapon_enchant()
        })?; // Confirmed
        inst.buffer.write(p.get_team())?; // Confirmed
        inst.buffer.write_i32(p.get_clan_crest_large_id())?;
        inst.buffer.write(p.is_noble())?; // Confirmed
        inst.buffer
            .write(p.is_hero() || (p.is_gm() && cfg.enable_gm_hero_aura))?; // Confirmed

        inst.buffer.write(p.is_fishing())?; // Confirmed
        if let Some(bait_location) = p.get_fishing_bait_location() {
            inst.buffer.write_i32(bait_location.x)?; // Confirmed
            inst.buffer.write_i32(bait_location.y)?; // Confirmed
            inst.buffer.write_i32(bait_location.z)?; // Confirmed
        } else {
            inst.buffer.write_i32(0)?;
            inst.buffer.write_i32(0)?;
            inst.buffer.write_i32(0)?;
        }

        inst.buffer.write_i32(p.get_name_color())?; // Confirmed
        inst.buffer.write_i32(p.get_location().heading)?; // Confirmed
        inst.buffer.write(p.get_pledge_class())?;
        inst.buffer.write_i16(p.get_pledge_type())?;
        inst.buffer.write_i32(p.get_title_color())?; // Confirmed
        inst.buffer.write(if let Some(cw) = p.get_cursed_weapon() {
            cw.get_lvl()
        } else {
            0
        })?;
        inst.buffer.write_i32(p.get_clan_reputation_score())?;
        inst.buffer.write_i32(p.get_transformation_display_id())?; // Confirmed
        inst.buffer.write_i32(p.get_agation_id())?; // Confirmed
        inst.buffer.write(0)?; // nPvPRestrainStatus

        inst.buffer.write_u32(p.get_cur_cp().to_u32_rounded()?)?; // Confirmed
        inst.buffer.write_u32(p.get_max_hp().to_u32_rounded()?)?; // Confirmed
        inst.buffer.write_u32(p.get_cur_hp().to_u32_rounded()?)?; // Confirmed
        inst.buffer.write_u32(p.get_max_mp().to_u32_rounded()?)?; // Confirmed
        inst.buffer.write_u32(p.get_cur_mp().to_u32_rounded()?)?; // Confirmed

        inst.buffer.write(0)?; // cBRLectureMark

        inst.buffer.write_u32(u32::try_from(
            p.get_abnoraml_visual_effects().len() + usize::from(p.is_gm()),
        )?)?; // Confirmed
        for af in p.get_abnoraml_visual_effects() {
            inst.buffer.write_u16(*af)?; // Confirmed
        }
        if p.is_gm() {
            inst.buffer.write_u16(AbnormalVisualEffect::Stealth)?;
        }

        inst.buffer.write(if p.is_hero() { 100 } else { 0 })?;
        inst.buffer.write(p.is_hair_accessory_enabled())?; // Hair accessory
        inst.buffer.write(p.get_ability_points_used())?; // Used Ability Points
        Ok(inst)
    }
}
#[cfg(test)]
mod test {}
