use crate::controller::GameController;
use l2_core::bitmask::BitMask;
use l2_core::config::gs::GSServerConfig;
use l2_core::data::classes::mapping::Class;
use l2_core::game_objects::cursed_weapon::CursedWeapon;
use l2_core::game_objects::player::user_info::UserInfoType;
use l2_core::game_objects::player::Player;
use l2_core::game_objects::zone::ZoneId;
use l2_core::shared_packets::write::SendablePacketBuffer;
use l2_core::traits::conversion::{ToU16Rounded, ToU32Rounded};
use macro_common::SendablePacket;

#[allow(unused)]
#[derive(Debug, Clone, SendablePacket)]
pub struct UserInfo {
    pub buffer: SendablePacketBuffer,
    block_size: u32,
    mask: BitMask,
    title: String,
}

#[allow(unused)]
impl UserInfo {
    const PACKET_ID: u8 = 0x32;
    const EX_PACKET_ID: Option<u16> = None;

    #[allow(clippy::too_many_lines)]
    pub async fn new(
        player: &Player,
        user_info_flags: BitMask,
        controller: &GameController,
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
            title_size = Self::get_title(player).len() * 2;
            block_size += u32::try_from(title_size)?;
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
        inst.buffer.write_bytes(inst.mask.flags())?;
        let is_cl = if let Some(clan_id) = player.char_model.clan_id {
            controller
                .clan_ally_manager
                .read()
                .await
                .is_clan_leader(player.char_model.id, clan_id)
        } else {
            false
        };
        if inst.mask.contains_mask(UserInfoType::Relation) {
            let relation = player.get_relation(is_cl).await;
            inst.buffer.write_u32(relation)?;
        }
        if inst.mask.contains_mask(UserInfoType::BasicInfo) {
            inst.write_basic_info(player, controller)?;
        }
        if inst.mask.contains_mask(UserInfoType::BaseStats) {
            inst.write_base_stats(player)?;
        }

        if inst.mask.contains_mask(UserInfoType::MaxHpCpMp) {
            inst.write_max_hp_cp_mp(player)?;
        }

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
            inst.write_color(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::InventoryLimit) {
            inst.write_inventory_limit(player)?;
        }
        if inst.mask.contains_mask(UserInfoType::TrueHero) {
            inst.write_true_hero(player)?;
        }
        Ok(inst)
    }

    fn get_title(player: &Player) -> String {
        if player.is_gm() || player.is_invincible() {
            "[Invincible]".to_string()
        } else {
            player.char_model.title.clone().unwrap_or_default()
        }
    }
    fn write_basic_info(
        &mut self,
        char_info: &Player,
        controller: &GameController,
    ) -> anyhow::Result<()> {
        let visible_name = char_info.get_visible_name();
        let visible_name_size = 16 + u16::try_from(visible_name.len())? * 2;
        self.buffer.write_u16(visible_name_size)?;
        self.buffer
            .write_sized_c_utf16le_string(Some(visible_name))?;
        self.buffer.write(char_info.is_gm())?;
        self.buffer.write_i8(char_info.char_model.race_id)?;
        self.buffer.write(char_info.char_model.is_female)?;
        let class_id = Class::try_from(char_info.char_model.class_id)?;
        self.buffer.write_u32(class_id.get_root().id)?;
        self.buffer
            .write_i32(i32::from(char_info.char_model.class_id))?;
        self.buffer.write_u8(char_info.char_model.level)?;
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
    fn write_max_hp_cp_mp(&mut self, char_info: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(14u16)?;
        self.buffer
            .write_u32(char_info.get_max_hp().to_u32_rounded()?)?;
        self.buffer
            .write_u32(char_info.get_max_mp().to_u32_rounded()?)?;
        self.buffer
            .write_u32(char_info.get_max_cp().to_u32_rounded()?)?;
        Ok(())
    }
    fn write_current_hp_mp_cp_exp_sp(
        &mut self,
        char_info: &Player,
        controller: &GameController,
    ) -> anyhow::Result<()> {
        let exp_table = &controller.exp_table;
        self.buffer.write_u16(38u16)?;
        self.buffer
            .write_u32(char_info.char_model.cur_hp.to_u32_rounded()?)?;
        self.buffer
            .write_u32(char_info.char_model.cur_mp.to_u32_rounded()?)?;
        self.buffer
            .write_u32(char_info.char_model.cur_cp.to_u32_rounded()?)?;
        self.buffer.write_i64(char_info.char_model.sp)?;
        self.buffer.write_i64(char_info.char_model.exp)?;
        let current_threshold = exp_table.get_exp(u8::try_from(char_info.char_model.level)?);
        let next_threshold =
            exp_table.get_exp_for_next_lvl(u8::try_from(char_info.char_model.level)?);
        let gained = char_info.char_model.exp - current_threshold;
        let total = next_threshold - current_threshold;
        #[allow(clippy::cast_precision_loss)]
        let left = gained as f64 / total as f64;
        self.buffer
            .write_f64(if left.is_nan() { 0.0 } else { left })?;
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
            .write(player.char_model.can_craft || player.has_skill(248));
        self.buffer.write(0);
        Ok(())
    }
    fn write_stats(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(56u16)?;
        // todo: in L2J it is always 40 even though it is checking the weapon
        self.buffer.write_u16(40u16)?;
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
    pub fn write_speed(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(18u16)?;
        let run_speed = (f64::from(player.get_run_speed())
            / player.get_movement_speed_multiplier())
        .to_u16_rounded()?;
        let walk_speed = (f64::from(player.get_walk_speed())
            / player.get_movement_speed_multiplier())
        .to_u16_rounded()?;
        let swim_run_speed = (f64::from(player.get_swim_run_speed())
            / player.get_movement_speed_multiplier())
        .to_u16_rounded()?;
        let swim_walk_speed = (f64::from(player.get_swim_walk_speed())
            / player.get_movement_speed_multiplier())
        .to_u16_rounded()?;

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

    pub fn write_clan(&mut self, player: &Player) -> anyhow::Result<()> {
        let title = Self::get_title(player);
        self.buffer
            .write_u16(32u16 + u16::try_from(title.len() * 2)?)?;
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
    pub fn write_social(&mut self, player: &Player, cfg: &GSServerConfig) -> anyhow::Result<()> {
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
        self.buffer.write_u32(player.char_model.vitality_points)?;
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
            .write(player.get_cursed_weapon().map_or(0, CursedWeapon::get_lvl))?;
        Ok(())
    }
    pub fn write_true_hero(&mut self, player: &Player) -> anyhow::Result<()> {
        self.buffer.write_u16(9u16)?;
        self.buffer.write_i32(0)?;
        self.buffer.write_u16(0u16)?;
        if player.is_hero() {
            self.buffer.write(100)?;
        } else {
            self.buffer.write(0)?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use entities::test_factories::factories::{char_factory, user_factory};
    use l2_core::shared_packets::common::SendablePacket;
    use l2_core::traits::ServerConfig;
    use sea_orm::JsonValue;
    use std::str::FromStr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;

    #[tokio::test]
    #[allow(clippy::too_many_lines)]
    async fn test_write_user_info() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let mut char = char_factory(&db_pool, |mut m| {
            m.name = "Adelante".to_string();
            m.user_id = user.id;
            m.is_female = true;
            m.level = 1;
            m.max_hp = 98.00;
            m.max_mp = 59.00;
            m.max_cp = 49.00;
            m.cur_hp = 98.00;
            m.cur_mp = 59.00;
            m.cur_cp = 49.00;
            m.race_id = 0;
            m.x = -90939;
            m.y = 248_138;
            m.z = -3563;
            m.access_level = 0;
            m.face = 1;
            m.class_id = 10;
            m.variables = JsonValue::from_str(
                r#"{"visualFaceId":1,"visualHairColorId":2,"visualHairStyleId":3, "hairAccessoryEnabled": true}"#
            ).unwrap();
            m
        })
        .await;

        char.id = 268_476_204;
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = GameController::from_config(cfg);
        let template = controller
            .class_templates
            .try_get_template(Class::try_from(char.class_id).unwrap())
            .unwrap();
        let player = Player::new(char, vec![], template.clone());
        let p = UserInfo::new(&player, UserInfoType::all(), &controller)
            .await
            .unwrap();
        assert_eq!(p.block_size, 393);
        assert_eq!(p.mask.flags(), &[0xFF, 0xFF, 0xFE]);
        let relation = player.get_relation(false).await;
        assert_eq!(relation, 0);
        let class_id = Class::try_from(10i8).unwrap();
        assert_eq!(class_id.get_root().id as u32, 10u32);
        assert_eq!(22, player.get_str());
        assert_eq!(27, player.get_con());
        assert_eq!(21, player.get_dex());
        assert_eq!(20, player.get_wit());
        assert_eq!(39, player.get_men());
        assert_eq!(41, player.get_int());
        assert_eq!(player.get_hair_style(), 3);
        assert_eq!(player.get_hair_color(), 2);
        assert_eq!(player.get_face(), 1);
        assert!(player.is_hair_accessory_enabled());
        let mut buff = p.get_buffer();
        assert_eq!(
            [
                50, //packet id
                44, 159, 0, 16, //cjar model id
                137, 1, 0, 0, //block size
                23, 0, //23
                255, 255, 254, //flags
                0, 0, 0, 0, //relation
                32, 0, // basic info block size
                8, 0, // Adelante length
                65, 0, 100, 0, 101, 0, 108, 0, 97, 0, 110, 0, 116, 0, 101, 0, 0, 0, //Adelante
                1, //is_gm
                10, 0, 0, 0, //root_id
                10, 0, 0, 0, //class_id
                1, // level
                18, 0, //base stats
                22, 0, //str
                21, 0, //dex
                27, 0, //con
                41, 0, //int
                20, 0, //wit
                39, 0, //men
                0, 0, //??
                0, 0, //??
                14, 0, //next block
                98, 0, 0, 0, //max hp
                59, 0, 0, 0, //max mp
                49, 0, 0, 0, //max cp
                38, 0, //next block
                98, 0, 0, 0, //cur hp
                59, 0, 0, 0, //cur mp
                49, 0, 0, 0, //cur cp
                0, 0, 0, 0, 0, 0, 0, 0, //exp
                0, 0, 0, 0, 0, 0, 0, 0, //sp
                0, 0, 0, 0, 0, 0, 0, 0, //left
                4, 0, //enchant block
                0, //weapon ++
                0, //min armor ++
                15, 0, //next block
                3, 0, 0, 0, //hair style
                2, 0, 0, 0, //hair color
                1, 0, 0, 0, //face
                1, 6, 0, //status block
                0, //mount type
                0, //private store type
                0, //can craft
                0, //?
                56, 0, //stats block
                40, 0, //weapon?
                3, 0, 0, 0, //p atack
                44, 1, 0, 0, //patack spd
                54, 0, 0, 0, //pdef
                23, 0, 0, 0, //evasion rate
                31, 0, 0, 0, //accuracy
                60, 0, 0, 0, //critical hit
                6, 0, 0, 0, //matack
                77, 1, 0, 0, //matack spd
                44, 1, 0, 0, //patack spd
                15, 0, 0, 0, //magic evasion rate
                36, 0, 0, 0, //mdef
                15, 0, 0, 0, //m accuracy
                50, 0, 0, 0, //m crit
                14, 0, //elementals block
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 18, 0, //
                197, 156, 254, 255, //x
                74, 201, 3, 0, //y
                21, 242, 255, 255, //z
                0, 0, 0, 0, //vehicle
                18, 0, //speed block
                124, 0, //run spd
                88, 0, //walk spd
                66, 0, //swim run
                66, 0, //swim walk
                0, 0, //flRun
                0, 0, //flWalk
                0, 0, //run fly
                0, 0, //walk fly
                18, 0, //multipliers block
                132, 16, 66, 8, 33, 132, 244, 63, // run multiplier
                239, 186, 141, 179, 189, 123, 242, 63, //another
                18, 0, // collision radius block
                0, 0, 0, 0, 0, 0, 26, 64, 0, 0, 0, 0, 0, 128, 54, 64, 5, 0, //atack elementals
                0, 0, 0, 32, 0, //
                0, 0, //title size
                0, 0, //pledge type
                0, 0, 0, 0, //clan_id
                0, 0, 0, 0, //clan crest large id
                0, 0, 0, 0, //clan crest id
                0, 0, 0, 0, //clan privs
                0, //is_cl
                0, 0, 0, 0, //ally id
                0, 0, 0, 0, //ally crest id
                0, //is matching room
                22, 0, //social block
                0, //pvp flag
                0, 0, 0, 0, //reputation
                0, // is_noble
                0, //is_hero || is_gm
                0, //pledge class
                0, 0, 0, 0, //pk kills
                0, 0, 0, 0, //pvp kills
                0, 0, //recs
                0, 0, //recs left
                15, 0, // slots block
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 6, 0, 0, 0, 0, 0, 0, 4, 0, 0, 1, 10,
                0, 255, 255, 255, 0, 162, 249, 236, 0, 9, 0, 0, 0, 0, 0, 80, 0, 0, 9, 0, 0, 0, 0,
                0, 0, 0, 0
            ],
            buff.get_data_mut(false)[2..]
        );
    }
}
