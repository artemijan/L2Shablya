use std::ptr::drop_in_place;
use crate::controller::Controller;
use crate::data::classes::mapping::Class;
use l2_core::bitmask::BitMask;
use l2_core::config::gs::GSServer;
use l2_core::game_objects::player::user_info::UserInfoType;
use l2_core::game_objects::player::Player;
use l2_core::game_objects::zone::ZoneId;
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

    fn write_basic_info(
        &mut self,
        char_info: &Player,
        controller: &Controller,
    ) -> anyhow::Result<()> {
        let visible_name = char_info.get_visible_name();
        let visible_name_size = u16::try_from(visible_name.len())? * 2;
        self.buffer.write_u16(visible_name_size)?;
        self.buffer
            .write_sized_c_utf16le_string(Some(visible_name))?;
        self.buffer.write_bool(char_info.is_gm())?;
        self.buffer.write_i8(char_info.char_model.race_id)?;
        self.buffer.write_bool(char_info.char_model.is_female)?;
        #[allow(clippy::cast_sign_loss)]
        let class_id = Class::try_from(char_info.char_model.class_id)?;
        let template = controller.class_templates.try_get_template(class_id)?;
        self.buffer.write_u32(class_id.get_root().id)?;
        self.buffer
            .write_i32(i32::from(char_info.char_model.class_id))?;
        self.buffer.write_i8(char_info.char_model.level)?;
        Ok(())
    }

    fn write_base_stats(&mut self, char_info: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(18u16)?;
        self.buffer.write_u16(char_info.get_str())?;
        self.buffer.write_u16(char_info.get_dex())?;
        self.buffer.write_u16(char_info.get_con())?;
        self.buffer.write_u16(char_info.get_int())?;
        self.buffer.write_u16(char_info.get_wit())?;
        self.buffer.write_u16(char_info.get_men())?;
        self.buffer.write_u16(0u16)?;
        self.buffer.write_u16(0u16)?;
        Ok(())
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn write_max_hp_cp_mp(&mut self, char_info: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(14u16)?;
        self.buffer
            .write_u32(char_info.get_max_hp().round() as u32)?;
        self.buffer
            .write_u32(char_info.get_max_mp().round() as u32)?;
        self.buffer
            .write_u32(char_info.get_max_cp().round() as u32)?;
        Ok(())
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn write_current_hp_mp_cp_exp_sp(
        &mut self,
        char_info: &Player,
        controller: &Controller,
    ) -> anyhow::Result<()> {
        let exp_table = &controller.exp_table;
        self.buffer.write_u16(38u16)?;
        self.buffer
            .write_u32(char_info.char_model.cur_hp.round() as u32)?;
        self.buffer
            .write_u32(char_info.char_model.cur_mp.round() as u32)?;
        self.buffer
            .write_u32(char_info.char_model.cur_cp.round() as u32)?;
        self.buffer.write_i64(char_info.char_model.sp)?;
        self.buffer.write_i64(char_info.char_model.exp)?;
        let current_threshold = exp_table.get_exp(char_info.char_model.level as u8);
        let next_threshold = exp_table.get_exp(char_info.char_model.level as u8);
        #[allow(clippy::cast_precision_loss)]
        self.buffer.write_f64(
            (char_info.char_model.exp as f64 - current_threshold as f64)
                / (next_threshold as f64 - current_threshold as f64),
        )?;
        Ok(())
    }

    fn write_enchant_level(&mut self, char_info: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(4u16)?;
        self.buffer.write(char_info.get_weapon_enchant())?;
        self.buffer.write(char_info.get_min_armor_enchant())?;
        Ok(())
    }
    fn write_appearance(&mut self, char_info: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(15u16)?;
        self.buffer.write_i32(char_info.get_hair_style())?;
        self.buffer.write_i32(char_info.get_hair_color())?;
        self.buffer.write_i32(char_info.get_face())?;
        self.buffer.write(char_info.is_hair_accessory_enabled())?;
        Ok(())
    }
    fn write_status(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(6u16)?;
        self.buffer.write(player.get_mount_type());
        self.buffer.write(player.get_private_store_type());
        self.buffer
            .write(player.char_model.can_craft.unwrap() || player.has_skill(248));
        self.buffer.write(0);
        Ok(())
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn write_stats(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(56u16)?;
        self.buffer
            .write_u16(player.get_weapon().map_or(20, |_| 40) as u16)?;
        self.buffer.write_u32(player.get_p_attack())?;
        self.buffer.write_u32(player.get_p_atk_spd())?;
        self.buffer.write_u32(player.get_p_def())?;
        self.buffer.write_u32(player.get_evasion_rate())?;
        self.buffer.write_u32(player.get_accuracy())?;
        self.buffer.write_u32(player.get_critical_hit())?;
        self.buffer.write_u32(player.get_m_atk())?;
        self.buffer.write_u32(player.get_m_atk_spd())?;
        self.buffer.write_u32(player.get_p_atk_spd()); // Seems like atk speed - 1
        self.buffer.write_u32(player.get_magic_evasion_rate())?;
        self.buffer.write_u32(player.get_m_def())?;
        self.buffer.write_u32(player.get_magic_accuracy())?;
        self.buffer.write_u32(player.get_m_critical_hit())?;
        Ok(())
    }
    pub fn write_elementals(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(14u16)?;
        self.buffer.write_u16(0u16)?;
        self.buffer.write_u16(0u16)?;
        self.buffer.write_u16(0u16)?;
        self.buffer.write_u16(0u16)?;
        self.buffer.write_u16(0u16)?;
        self.buffer.write_u16(0u16)?;
        Ok(())
    }
    pub fn write_position(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(18u16)?;
        self.buffer.write_i32(player.get_x())?;
        self.buffer.write_i32(player.get_y())?;
        self.buffer.write_i32(player.get_z())?;
        self.buffer
            .write_i32(player.get_vehicle_object_id().unwrap_or(0))?;
        Ok(())
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn write_speed(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(18u16)?;
        let run_speed: u16 = (f64::from(player.get_run_speed())
            / player.get_movement_speed_multiplier())
        .round() as u16;
        let walk_speed: u16 = (f64::from(player.get_walk_speed())
            / player.get_movement_speed_multiplier())
        .round() as u16;
        let swim_run_speed: u16 = (f64::from(player.get_swim_run_speed())
            / player.get_movement_speed_multiplier())
        .round() as u16;
        let swim_walk_speed: u16 = (f64::from(player.get_swim_walk_speed())
            / player.get_movement_speed_multiplier())
        .round() as u16;

        self.buffer.write_u16(run_speed)?;
        self.buffer.write_u16(walk_speed)?;
        self.buffer.write_u16(swim_run_speed)?;
        self.buffer.write_u16(swim_walk_speed)?;
        self.buffer.write_u16(0u16)?; //flRunSpeed
        self.buffer.write_u16(0u16)?; //flWalkSpeed
        if player.is_flying() {
            self.buffer.write_u16(run_speed)?;
            self.buffer.write_u16(walk_speed)?;
        } else {
            self.buffer.write_u16(0u16)?;
            self.buffer.write_u16(0u16)?;
        }
        Ok(())
    }
    pub fn write_multiplier(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(18u16)?;
        self.buffer
            .write_f64(player.get_movement_speed_multiplier())?;
        self.buffer.write_f64(player.get_p_atk_spd_multiplier())?;
        Ok(())
    }
    pub fn write_collision(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(18u16)?;
        self.buffer.write_f64(player.get_collision_radius())?;
        self.buffer.write_f64(player.get_collision_height())?;
        Ok(())
    }
    pub fn write_attack_elementals(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(5u16)?;
        self.buffer.write(0)?;
        self.buffer.write_u16(0u16)?;
        Ok(())
    }
    #[allow(clippy::cast_possible_truncation)]
    pub fn write_clan(&mut self, player: &Player) -> anyhow::Result<()> {
        let title = if player.is_gm() || player.is_invincible() {
            "[Invincible]".to_string()
        } else {
            player.char_model.title.clone().unwrap_or_default()
        };
        self.buffer.write_u16(32u16 + (title.len() * 2) as u16)?;
        self.buffer.write_sized_c_utf16le_string(Some(&title))?;
        self.buffer.write_u16(player.get_pledge_type())?;
        self.buffer
            .write_i32(player.char_model.clan_id.unwrap_or(0))?;
        self.buffer.write_i32(player.get_clan_crest_large_id())?;
        self.buffer.write_i32(player.get_clan_crest_id())?;
        self.buffer
            .write_i32(player.char_model.clan_privs.unwrap_or(0))?;
        self.buffer.write(player.is_clan_leader())?;
        self.buffer.write_i32(player.get_ally_id())?;
        self.buffer.write_i32(player.get_ally_crest_id())?;
        self.buffer.write(player.is_in_matching_room())?;
        Ok(())
    }
    pub fn write_social(&mut self, player: &Player, cfg: &GSServer) -> anyhow::Result<()> {
        self.buffer.write_u16(22u16)?;
        self.buffer.write(player.get_pvp_flag())?;
        self.buffer.write_i32(player.char_model.reputation)?;
        self.buffer.write(player.is_noble())?;
        self.buffer
            .write(player.is_hero() || player.is_gm() && cfg.enable_gm_hero_aura)?;
        self.buffer.write(player.get_pledge_class())?;
        self.buffer.write_i32(player.char_model.pk_kills)?;
        self.buffer.write_i32(player.char_model.pvp_kills)?;
        self.buffer.write_u16(player.get_recommendations_left())?;
        self.buffer.write_u16(player.get_recommendations_have())?;
        Ok(())
    }
    pub fn write_vita_fame(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(15u16)?;
        self.buffer.write_i32(player.char_model.vitality_points)?;
        self.buffer.write(0)?; //Vitality bonus
        self.buffer.write_i32(player.char_model.fame)?;
        self.buffer.write_i32(player.char_model.rb_points)?;
        Ok(())
    }
    pub fn write_slots(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(9u16)?;
        self.buffer.write(player.inventory.get_talisman_slots())?;
        self.buffer
            .write(player.inventory.get_brooch_jewel_slots())?;
        self.buffer.write(player.team)?;
        self.buffer.write(0)?; // (1 = Red, 2 = White, 3 = White Pink) dotted ring on the floor
        self.buffer.write(0)?;
        self.buffer.write(0)?;
        self.buffer.write(0)?;
        Ok(())
    }
    pub fn write_movements(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(4u16)?;
        let movement = if player.is_inside_zone(ZoneId::Water) {
            1
        } else if player.is_flying() {
            2
        } else {
            0
        };
        self.buffer.write(movement)?;
        self.buffer.write(player.is_running())?;
        Ok(())
    }
    pub fn write_color(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(10u16)?;
        self.buffer.write_i32(player.appearance.get_name_color())?;
        self.buffer.write_i32(player.appearance.get_title_color())?;
        Ok(())
    }
    pub fn write_inventory_limit(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(9u16)?;
        self.buffer.write_u16(0u16)?;
        self.buffer.write_u16(0u16)?;
        self.buffer.write_u16(player.inventory.get_limit())?;
        self.buffer
            .write(player.get_cursed_weapon().map_or(0, |w| w.get_lvl()))?;
        Ok(())
    }
    pub fn write_true_hero(&mut self,player: &Player)-> anyhow::Result<()> {
        self.buffer.write_u16(9u16)?;
        self.buffer.write_i32(0)?;
        self.buffer.write_u16(0u16)?;
        if player.is_hero() {
            self.buffer.write(100)?;
        }else { 
            self.buffer.write(0)?;
        }
        Ok(())
    }
    pub async fn new(
        player: &Player,
        user_info_flags: BitMask,
        controller: &Controller,
    ) -> anyhow::Result<Self> {
        //todo: check is subclass locked
        let mut block_size = 5;
        block_size += UserInfoType::calculate_block_size(&user_info_flags);
        let mut visible_name_size = 0;
        let exp_table = &controller.exp_table;
        let cfg = controller.get_cfg();
        if user_info_flags.contains_mask(UserInfoType::BasicInfo) {
            visible_name_size = u16::try_from(player.get_visible_name().len())? * 2;
            block_size += u32::from(visible_name_size);
        }
        let mut title_size = 0;
        if user_info_flags.contains_mask(UserInfoType::Clan) {
            title_size = u16::try_from(
                player
                    .char_model
                    .title
                    .as_ref()
                    .unwrap_or(&String::new())
                    .len(),
            )? * 2;
            block_size += u32::from(title_size);
        }
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            block_size,
            mask: user_info_flags,
            title: String::new(),
        };
        inst.buffer.write(Self::PACKET_ID)?;
        inst.buffer.write_i32(player.char_model.id)?;
        inst.buffer.write_u32(inst.block_size)?;
        inst.buffer.write_u16(23u16)?;
        inst.buffer.write_bytes(inst.mask.bytes())?;
        let is_cl = controller
            .clan_ally_manager
            .read()
            .await
            .is_clan_leader(player.char_model.id, player.char_model.clan_id.unwrap());
        if inst.mask.contains_mask(UserInfoType::Relation) {
            inst.buffer.write_u32(player.get_relation(is_cl).await)?;
        }
        if inst.mask.contains_mask(UserInfoType::BasicInfo) {
            inst.write_basic_info(player, controller)?;
        }
        if inst.mask.contains_mask(UserInfoType::BaseStats) {
            inst.write_base_stats(player)?;
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        if inst.mask.contains_mask(UserInfoType::MaxHpCpMp) {
            inst.write_max_hp_cp_mp(player)?;
        }
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        if inst.mask.contains_mask(UserInfoType::CurrentHpMpCpExpSp) {
            inst.write_current_hp_mp_cp_exp_sp(player, controller)?;
        }
        if inst.mask.contains_mask(UserInfoType::EnchantLevel) {
            inst.write_enchant_level(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Appearance) {
            inst.write_appearance(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Status) {
            inst.write_status(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Stats) {
            inst.write_stats(player)?;
        }
        if (inst.mask.contains_mask(UserInfoType::Elementals)) {
            inst.write_elementals(player)?;
        }
        if (inst.mask.contains_mask(UserInfoType::Position)) {
            inst.write_position(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Speed) {
            inst.write_speed(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Multiplier) {
            inst.write_multiplier(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::ColRadiusHeight) {
            inst.write_collision(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::AtkElemental) {
            inst.write_attack_elementals(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Clan) {
            inst.write_clan(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Social) {
            inst.write_social(player, &cfg)?;
        }
        if inst.mask.contains_mask(UserInfoType::VitaFame) {
            inst.write_vita_fame(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Slots) {
            inst.write_slots(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Movements) {
            inst.write_movements(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::Color) {
            inst.write_color(player)?
        }
        if inst.mask.contains_mask(UserInfoType::InventoryLimit) {
            inst.write_inventory_limit(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::TrueHero){
            inst.write_true_hero(player)?;
        }
        Ok(inst)
    }
}
