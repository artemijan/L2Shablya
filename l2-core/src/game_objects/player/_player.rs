use crate::game_objects::cursed_weapon::CursedWeapon;
use crate::game_objects::player::appearance::Appearance;
use crate::game_objects::player::inventory::Inventory;
use crate::game_objects::player::paper_doll::PaperDoll;
use crate::game_objects::player::party::Party;
use crate::game_objects::player::vars::CharVariables;
use crate::game_objects::player::{PlayerMacro, TeleportBookmark};
use crate::game_objects::race::Race;
use crate::game_objects::zone::ZoneId;
use chrono::Utc;
use entities::entities::{character, item};
use serde_json::Value;
use crate::game_objects::item::ItemObject;

#[repr(u8)]
#[derive(Clone, Debug, Copy)]
pub enum Team {
    None = 0,
    Blue = 1,
    Red = 2,
}
impl From<u8> for Team {
    fn from(team: u8) -> Self {
        match team {
            1 => Team::Red,
            2 => Team::Blue,
            _ => Team::None,
        }
    }
}
impl From<Team> for u8 {
    fn from(team: Team) -> Self {
        team as u8
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub char_model: character::Model,
    pub items: Vec<ItemObject>,
    pub paperdoll: [[i32; 4]; 33],
    pub party: Option<Party>,
    pub inventory: Inventory,
    pub appearance: Appearance,
    pub team: Team,
    pub is_in_siege: bool,
}

#[allow(clippy::missing_errors_doc)]
impl Player {
    #[must_use]
    pub fn new(char_model: character::Model, items: Vec<item::Model>) -> Self {
        let paperdoll = PaperDoll::restore_visible_inventory(&items);
        Self {
            char_model,
            items: ItemObject::from_items(items),
            party: None,
            paperdoll,
            team: Team::None,
            is_in_siege: false,
            appearance: Appearance,
            inventory: Inventory,
        }
    }
    #[must_use]
    pub fn get_visible_name(&self) -> &str {
        &self.char_model.name
    }
    #[must_use]
    pub fn get_macros(&self) -> &Vec<PlayerMacro> {
        //todo: implement me
        static EMPTY: Vec<PlayerMacro> = Vec::new();
        &EMPTY
    }
    #[must_use]
    pub fn get_henna_slots(&self) -> u32 {
        //todo: implement me
        3
    }
    #[must_use]
    pub fn get_str(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_con(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_dex(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_men(&self) -> u8 {
        //todo: implement me
        0
    }

    #[must_use]
    pub fn get_int(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_wit(&self) -> u8 {
        //todo: implement me
        0
    }

    #[must_use]
    pub fn is_gm(&self) -> bool {
        // todo: implement me
        self.char_model.access_level >= 0
    }

    #[must_use]
    pub fn is_invincible(&self) -> bool {
        // todo: implement me
        false
    }
    #[must_use]
    pub fn get_pvp_flag(&self) -> bool {
        // todo: implement me
        false
    }
    #[must_use]
    pub fn is_noble(&self) -> bool {
        // todo: implement me
        false
    }
    #[must_use]
    pub fn is_hero(&self) -> bool {
        // todo: implement me
        false
    }
    #[must_use]
    pub fn get_pledge_type(&self) -> u16 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_pledge_class(&self) -> u8 {
        //todo: implement me
        0
    }

    #[must_use]
    pub fn get_recommendations_left(&self) -> u16 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_recommendations_have(&self) -> u16 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn is_inside_zone(&self, zone: ZoneId) -> bool {
        //todo: implement me
        false
    }

    #[must_use]
    pub fn get_clan_crest_large_id(&self) -> i32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_clan_crest_id(&self) -> i32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_ally_id(&self) -> i32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_ally_crest_id(&self) -> i32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn is_in_matching_room(&self) -> bool {
        false
    }

    #[must_use]
    pub fn is_clan_leader(&self) -> bool {
        //todo: implement me
        false
    }
    #[must_use]
    pub fn get_visible_name_length(&self) -> usize {
        self.char_model.name.len() * 2
    }
    pub fn set_party(&mut self, party: Option<Party>) {
        self.party = party;
    }
    #[must_use]
    pub fn paper_doll_item_id(&self, slot: PaperDoll) -> Option<i32> {
        Some(self.paperdoll.get(slot as usize)?[1])
    }

    pub fn try_get_paper_doll_item_id(&self, slot: PaperDoll) -> anyhow::Result<i32> {
        self.paper_doll_item_id(slot)
            .ok_or(anyhow::anyhow!("No paper doll item id at slot {slot:?}"))
    }

    #[must_use]
    pub fn is_dead(&self) -> bool {
        self.char_model.cur_hp <= 0.5
    }

    #[must_use]
    pub fn get_item(&self, item_obj_id: i32) -> Option<&ItemObject> {
        self.items.iter().find(|i| i.item_model.id == item_obj_id)
    }

    #[must_use]
    pub fn get_weapon(&self) -> Option<&ItemObject> {
        if let Some(r_id) = self.get_paper_doll_object_id(PaperDoll::RHand) {
            return self.get_item(r_id);
        }
        None
    }
    #[must_use]
    pub fn get_movement_speed_multiplier(&self) -> f64 {
        //todo: implement me
        1.0
    }
    #[must_use]
    pub fn get_run_speed(&self) -> u16 {
        //todo: implement me
        150
    }
    #[must_use]
    pub fn get_walk_speed(&self) -> u16 {
        //todo: implement me
        130
    }

    #[must_use]
    pub fn is_flying(&self) -> bool {
        //todo: implement me
        false
    }

    #[must_use]
    pub fn get_cursed_weapon(&self) -> Option<&dyn CursedWeapon> {
        //todo: implement me
        None
    }
    #[must_use]
    pub fn is_running(&self) -> bool {
        //todo: implement me
        true
    }

    #[must_use]
    pub fn get_swim_run_speed(&self) -> u16 {
        //todo: implement me
        99
    }
    #[must_use]
    pub fn get_swim_walk_speed(&self) -> u16 {
        //todo: implement me
        89
    }
    #[must_use]
    pub fn get_mount_type(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn has_skill(&self, skill_id: u32) -> bool {
        //todo: implement me
        false
    }
    #[must_use]
    pub fn has_inventory_block(&self) -> bool {
        //todo: implement me
        false
    }
    #[must_use]
    pub fn get_private_store_type(&self) -> u8 {
        //todo: implement me
        0
    }

    #[must_use]
    pub fn get_p_attack(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_p_atk_spd(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_p_atk_spd_multiplier(&self) -> f64 {
        //todo: implement me
        0.0
    }
    #[must_use]
    pub fn get_collision_radius(&self) -> f64 {
        //todo: implement me
        0.0
    }
    #[must_use]
    pub fn get_collision_height(&self) -> f64 {
        //todo: implement me
        0.0
    }
    #[must_use]
    pub fn get_p_def(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_evasion_rate(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_accuracy(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_critical_hit(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_m_atk(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_m_atk_spd(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_magic_evasion_rate(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_m_def(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_magic_accuracy(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_m_critical_hit(&self) -> u32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_x(&self) -> i32 {
        //todo: implement me
        self.char_model.x
    }
    #[must_use]
    pub fn get_y(&self) -> i32 {
        //todo: implement me
        self.char_model.x
    }
    #[must_use]
    pub fn get_z(&self) -> i32 {
        //todo: implement me
        self.char_model.x
    }
    #[must_use]
    pub fn get_vehicle_object_id(&self) -> Option<i32> {
        //todo: implement me
        None
    }
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_weapon_enchant(&self) -> u8 {
        self.get_weapon().map_or(0, |w| w.item_model.enchant_level) as u8
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn get_min_armor_enchant(&self) -> u8 {
        //todo: implement me
        0
    }

    pub fn try_get_race(&self) -> anyhow::Result<Race> {
        Race::try_from(self.char_model.race_id)
    }

    #[must_use]
    pub fn get_paper_doll_object_id(&self, slot: PaperDoll) -> Option<i32> {
        Some(self.paperdoll.get(slot as usize)?[0])
    }

    #[must_use]
    pub fn get_paper_doll_visual_id(&self, slot: PaperDoll) -> Option<i32> {
        Some(self.paperdoll.get(slot as usize)?[3])
    }
    pub fn try_get_paper_doll_visual_id(&self, slot: PaperDoll) -> anyhow::Result<i32> {
        self.get_paper_doll_visual_id(slot)
            .ok_or(anyhow::anyhow!("No paperdoll at slot {slot:?}"))
    }
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_enchant_effect(&self, slot: PaperDoll) -> i16 {
        self.paperdoll[slot as usize][3] as i16
    }
    #[must_use]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_enchant_effect_as_byte(&self, slot: PaperDoll) -> u8 {
        let effect = self.get_enchant_effect(slot);
        if effect > 127 { 127 } else { effect as u8 }
    }

    pub fn get_hair_style(&self) -> i32 {
        self.char_model
            .variables
            .get(CharVariables::VisualHairStyleId.as_key())
            .and_then(Value::as_i64) // Convert to i64 (Json numbers are i64 in Serde)
            .and_then(|v| v.try_into().ok()) // Convert to i32 safely
            .unwrap_or(0)
    }

    pub fn get_hair_color(&self) -> i32 {
        self.char_model
            .variables
            .get(CharVariables::VisualHairColorId.as_key())
            .and_then(Value::as_i64) // Convert to i64 (Json numbers are i64 in Serde)
            .and_then(|v| v.try_into().ok()) // Convert to i32 safely
            .unwrap_or(0)
    }

    pub fn get_face(&self) -> i32 {
        self.char_model
            .variables
            .get(CharVariables::VisualFaceId.as_key())
            .and_then(Value::as_i64) // Convert to i64 (Json numbers are i64 in Serde)
            .and_then(|v| v.try_into().ok()) // Convert to i32 safely
            .unwrap_or(0)
    }

    #[must_use]
    pub fn get_max_hp(&self) -> f64 {
        self.char_model.max_hp
    }
    
    #[must_use]
    pub fn get_max_mp(&self) -> f64 {
        self.char_model.max_mp
    }

    #[must_use]
    pub fn get_max_cp(&self) -> f64 {
        self.char_model.max_cp
    }
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn get_delete_timer(&self) -> i32 {
        if let Some(delete_at) = self.char_model.delete_at {
            let now = Utc::now();
            let delta = delete_at.with_timezone(&Utc).signed_duration_since(now);
            let time_left = delta.num_seconds() as i32;
            if time_left >= 0 {
                return time_left;
            }
        }
        0
    }
    #[must_use]
    pub async fn get_relation(&self, is_clan_leader: bool) -> u32 {
        let mut relation = 0;
        if let Some(pt) = self.party.as_ref() {
            relation |= 0x08;
            if pt.get_leader_id().await == self.char_model.id {
                relation |= 0x10;
            }
        }
        if self.char_model.clan_id.is_some() {
            relation |= 0x20;

            if is_clan_leader {
                relation |= 0x40;
            }
        }
        if self.is_in_siege {
            relation |= 0x80;
        }
        relation
    }
    #[must_use]
    pub fn get_vitality_used(&self) -> i32 {
        self.char_model
            .variables
            .get(CharVariables::VitalityItemsUsed.as_key())
            .and_then(Value::as_i64)
            .and_then(|v| v.try_into().ok())
            .unwrap_or(0)
    }
    #[must_use]
    pub fn get_vitality_bonus(&self) -> i32 {
        // todo: implement me
        0
    }
    #[must_use]
    pub fn get_teleport_bookmarks(&self,)->&Vec<TeleportBookmark>{
        //todo: implement me
        static EMPTY: Vec<TeleportBookmark> = Vec::new();
        &EMPTY
    }
    #[must_use]
    pub fn get_bookmark_slot(&self) -> i32 {
        // todo: implement me
        0
    }

    #[must_use]
    pub fn is_hair_accessory_enabled(&self) -> bool {
        self.char_model
            .variables
            .get(CharVariables::HairAccessoryEnabled.as_key())
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }
    #[must_use]
    pub fn get_transform_id(&self) -> i32 {
        i32::from(self.char_model.transform_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use entities::dao::item::{ItemVariables, ItemVariations, LocType};
    use entities::test_factories::factories::{char_factory, item_factory, user_factory};
    use serde_json::json;
    use test_utils::utils::get_test_db;

    #[test]
    fn test_get_paperdoll_ordered_ids() {
        let ids = PaperDoll::ordered_ids();
        let expected_ids = [
            PaperDoll::Under,
            PaperDoll::Rear,
            PaperDoll::Lear,
            PaperDoll::Neck,
            PaperDoll::RFinger,
            PaperDoll::LFinger,
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
            PaperDoll::RBracelet,
            PaperDoll::LBracelet,
            PaperDoll::Deco1,
            PaperDoll::Deco2,
            PaperDoll::Deco3,
            PaperDoll::Deco4,
            PaperDoll::Deco5,
            PaperDoll::Deco6,
            PaperDoll::Belt,
            PaperDoll::Brooch,
            PaperDoll::BroochJewel1,
            PaperDoll::BroochJewel2,
            PaperDoll::BroochJewel3,
            PaperDoll::BroochJewel4,
            PaperDoll::BroochJewel5,
            PaperDoll::BroochJewel6,
        ];
        assert_eq!(ids, expected_ids);
    }
    #[test]
    fn test_get_paperdoll_visual_ids() {
        let ids = PaperDoll::visual_ids();
        let expected_ids = [
            PaperDoll::RHand,
            PaperDoll::LHand,
            PaperDoll::Gloves,
            PaperDoll::Chest,
            PaperDoll::Legs,
            PaperDoll::Feet,
            PaperDoll::RHand,
            PaperDoll::Hair,
            PaperDoll::Hair2,
        ];
        assert_eq!(ids, expected_ids);
    }
    #[test]
    fn test_valid_race_values() {
        use Race::*;
        let test_cases = vec![
            (0, Human),
            (1, Elf),
            (2, DarkElf),
            (3, Orc),
            (4, Dwarf),
            (5, Kamael),
            (6, Ertheia),
            (7, Animal),
            (8, Beast),
            (9, Bug),
            (10, CastleGuard),
            (11, Construct),
            (12, Demonic),
            (13, Divine),
            (14, Dragon),
            (15, Elemental),
            (16, Etc),
            (17, Fairy),
            (18, Giant),
            (19, Humanoid),
            (20, Mercenary),
            (21, None),
            (22, Plant),
            (23, SiegeWeapon),
            (24, Undead),
        ];

        for (input, expected) in test_cases {
            assert_eq!(Race::try_from(input).unwrap(), expected);
        }
    }
    #[test]
    fn test_invalid_race_values() {
        let invalid_values = vec![-1, 25, 100, i32::MAX, i32::MIN];

        for &value in &invalid_values {
            assert!(
                Race::try_from(value).is_err(),
                "Expected error for value: {value}"
            );
        }
    }

    #[tokio::test]
    async fn test_get_char_info() {
        let db_pool = get_test_db().await;
        let user = user_factory(&db_pool, |u| u).await;
        let now = Utc::now();
        let char = char_factory(&db_pool, |mut ch| {
            ch.user_id = user.id;
            ch.max_hp = 300f64;
            ch.max_mp = 400f64;
            ch.max_cp = 500f64;
            ch.variables = json!({
                CharVariables::VitalityItemsUsed.as_key(): 10,
                CharVariables::VisualFaceId.as_key(): 3,
                CharVariables::HairAccessoryEnabled.as_key(): false,
                CharVariables::VisualHairColorId.as_key(): 4,
                CharVariables::VisualHairStyleId.as_key(): 3,
            });
            ch.delete_at = Some(now.into());
            ch
        })
        .await;
        let item1 = item_factory(&db_pool, |mut it| {
            it.owner = char.id;
            it.variations = json!({
                ItemVariations::MineralId.as_key(): 9,
                ItemVariations::Option1.as_key(): 3,
                ItemVariations::Option2.as_key(): 5,
            });
            it
        })
        .await;
        let item2 = item_factory(&db_pool, |mut it| {
            it.owner = char.id;
            it.item_id = 2;
            it.count = 1;
            it.enchant_level = 0;
            it.variables = json!({
               ItemVariables::VisualId.as_key(): 3
            });
            it.time_of_use = 0;
            it.loc = LocType::Paperdoll;
            it.loc_data = PaperDoll::Hair as i32;
            it
        })
        .await;
        let items = vec![item1, item2];
        let char_info = Player::new(char, items);
        let weapon = char_info.get_weapon().unwrap();
        let augmentation = weapon.item_model.get_augmentation().unwrap();
        assert_eq!((9, 3, 5), augmentation);
        assert_eq!(weapon.item_model.id, 1);
        assert!(!char_info.is_hair_accessory_enabled());
        assert_eq!(char_info.get_hair_color(), 4);
        assert_eq!(char_info.get_face(), 3);
        assert_eq!(char_info.get_max_hp().to_bits(), 300f64.to_bits());
        assert_eq!(char_info.get_max_mp().to_bits(), 400f64.to_bits());
        assert_eq!(char_info.get_max_cp().to_bits(), 500f64.to_bits());
        assert_eq!(
            char_info
                .try_get_paper_doll_visual_id(PaperDoll::Hair)
                .unwrap(),
            3
        );
        assert_eq!(
            char_info
                .try_get_paper_doll_item_id(PaperDoll::RHand)
                .unwrap(),
            2
        );
        assert_eq!(char_info.get_enchant_effect(PaperDoll::RHand), 0);
        assert_eq!(char_info.get_hair_style(), 3);
        assert_eq!(char_info.get_transform_id(), 2);
        assert_eq!(char_info.get_delete_timer(), 0);
        assert_eq!(char_info.get_vitality_used(), 10);
    }
}
