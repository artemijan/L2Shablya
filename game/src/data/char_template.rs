use crate::data::classes::mapping::Class;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use tracing::info;

#[derive(Clone, Debug)]
pub struct ClassTemplates {
    templates: HashMap<Class, CharTemplate>,
}

impl ClassTemplates {
    const DATA_DIR: &'static str = "config/data/stats/chars/base_stats";
    pub fn load() -> Self {
        let dir_entries = fs::read_dir(Self::DATA_DIR).unwrap_or_else(|e| {
            panic!("Can't open {} to read class templates. {e}", Self::DATA_DIR)
        });
        let mut result = Self {
            templates: HashMap::new(),
        };
        for dir_entry in dir_entries {
            let path = dir_entry.expect("Can not read dir entry").path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                let file = fs::read_to_string(path).expect("Can not read file");
                let inst: CharTemplate =
                    serde_yaml::from_str(&file).expect("Can't read experience table");
                if let Some(i) = result.templates.insert(inst.class_id, inst) {
                    panic!("Duplicate template id: {:?}", i.class_id);
                }
            }
        }
        info!("Loaded {} class templates.", result.templates.len());
        for t in Self::registration_classes() {
            assert!(
                result.templates.contains_key(t),
                "Missing class template for class id: {t:?}"
            );
        }
        result
    }
    pub fn get_template(&self, template_id: Class) -> Option<CharTemplate> {
        self.templates.get(&template_id).cloned()
    }

    fn registration_classes() -> &'static [Class] {
        &[
            Class::Fighter,
            Class::Mage,
            Class::ElvenFighter,
            Class::ElvenMage,
            Class::DarkFighter,
            Class::DarkMage,
            Class::OrcFighter,
            Class::OrcMage,
            Class::DwarvenFighter,
        ]
    }
    pub fn get_available_templates_for_registration(&self) -> Vec<CharTemplate> {
        Self::registration_classes()
            .iter()
            .map(|i| self.templates.get(i).unwrap().clone())
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharTemplate {
    pub class_id: Class,
    pub static_data: CharTemplateStaticData,
    pub lvl_up_gain_data: Vec<LvlUpGainData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharTemplateStaticData {
    pub base_int: i32,
    pub base_str: i32,
    pub base_con: i32,
    pub base_men: i32,
    pub base_dex: i32,
    pub base_wit: i32,
    pub physical_abnormal_resist: i32,
    pub magic_abnormal_resist: i32,
    pub creation_points: Vec<Point>,
    pub base_p_atk: i32,
    pub base_crit_rate: i32,
    pub base_m_crit_rate: i32,
    pub base_atk_type: BaseAtkType,
    pub base_p_atk_spd: i32,
    pub base_m_atk_spd: i32,
    pub base_p_def: BasePDef,
    pub base_m_atk: i32,
    pub base_m_def: BaseMDef,
    pub base_can_penetrate: i32,
    pub base_atk_range: i32,
    pub base_dam_range: BaseDamRange,
    pub base_rnd_dam: i32,
    pub base_move_spd: BaseMoveSpeed,
    pub base_breath: i32,
    pub base_safe_fall: i32,
    pub collision_male: CharCollision,
    pub collision_female: CharCollision,
}
#[derive(Debug, Clone, Deserialize)]
pub struct LvlUpGainData {
    pub lvl: u8,
    pub hp: f32,
    pub mp: f32,
    pub cp: f32,
    pub hp_regen: f32,
    pub mp_regen: f32,
    pub cp_regen: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct CharCollision {
    pub radius: f32,
    pub height: f32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct BaseMoveSpeed {
    pub walk: i32,
    pub run: i32,
    pub slow_swim: i32,
    pub fast_swim: i32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct BaseDamRange {
    pub vertical_direction: i32,
    pub horizontal_direction: i32,
    pub distance: i32,
    pub width: i32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct BaseMDef {
    pub r_ear: i32,
    pub l_ear: i32,
    pub r_finger: i32,
    pub l_finger: i32,
    pub neck: i32,
}
#[derive(Debug, Clone, Deserialize)]
pub struct BasePDef {
    pub chest: i32,
    pub legs: i32,
    pub head: i32,
    pub feet: i32,
    pub gloves: i32,
    pub underwear: i32,
    pub cloak: i32,
}

impl BasePDef {
    pub fn total(&self) -> i32 {
        self.chest + self.legs + self.head + self.feet + self.gloves + self.underwear + self.cloak
    }
}

impl BaseMDef {
    pub fn total(&self) -> i32 {
        self.r_ear + self.l_ear + self.l_finger + self.neck
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BaseAtkType {
    Fist,
}
#[derive(Debug, Clone, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
