use crate::data::base_stat::{BaseStat, CreatureParameter};
use crate::data::classes::mapping::Class;
use anyhow::bail;
use entities::dao::char_info::CharVariables;
use entities::entities::character;
use l2_core::config::traits::{LoadFileHandler, Loadable};
use macro_common::config_dir;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Deserializer};
use serde_json::json;
use std::collections::HashMap;
use tracing::info;

#[derive(Clone, Debug, Default)]
#[config_dir(path = "config/data/stats/chars/base_stats", post_load)]
pub struct ClassTemplates {
    templates: HashMap<Class, CharTemplate>,
}

impl Loadable for ClassTemplates {
    fn post_load(&self) {
        info!("Loaded {} class templates.", self.templates.len());
        for t in Self::registration_classes() {
            assert!(
                self.templates.contains_key(t),
                "Missing class template for class id: {t:?}"
            );
        }
    }
}

impl LoadFileHandler for ClassTemplates {
    type TargetConfigType = CharTemplate;
    fn for_each(&mut self, item: Self::TargetConfigType) {
        if let Some(i) = self.templates.insert(item.class_id, item) {
            panic!("Duplicate template id: {:?}", i.class_id);
        }
    }
}
impl ClassTemplates {
    #[must_use]
    pub fn get_template(&self, template_id: Class) -> Option<&CharTemplate> {
        self.templates.get(&template_id)
    }

    /// # Errors
    /// - when template is not found
    pub fn try_get_template(&self, template_id: Class) -> anyhow::Result<&CharTemplate> {
        self.templates.get(&template_id).ok_or(anyhow::anyhow!(
            "Invalid class template: {:?}.",
            template_id
        ))
    }
    #[must_use]
    pub fn has_template(&self, template_id: Class) -> bool {
        self.templates.contains_key(&template_id)
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
    /// # Panics
    /// - when registration classes mismatch with available classes
    #[must_use]
    pub fn get_available_templates_for_registration(&self) -> Vec<&CharTemplate> {
        Self::registration_classes()
            .iter()
            .map(|i| {
                self.templates.get(i).expect(
                    r"
                         It seems like you misconfigured registration templates,
                         so they contain classes that are not available at all.",
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharTemplate {
    pub class_id: Class,
    pub static_data: CharTemplateStaticData,

    #[serde(deserialize_with = "deserialize_lvl_up_gain_data")]
    pub lvl_up_gain_data: HashMap<u8, LvlUpGainData>,
}

fn deserialize_lvl_up_gain_data<'de, D>(
    deserializer: D,
) -> Result<HashMap<u8, LvlUpGainData>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize into a Vec<LvlUpGainData>
    let vec: Vec<LvlUpGainData> = Deserialize::deserialize(deserializer)?;

    // Convert the Vec into a HashMap
    let map: HashMap<u8, LvlUpGainData> = vec.into_iter().map(|data| (data.lvl, data)).collect();

    Ok(map)
}

impl CharTemplate {
    /// # Errors
    /// - when something wrong with templates
    #[allow(clippy::cast_sign_loss, clippy::similar_names)]
    pub fn initialize_character(
        &self,
        target: &mut character::Model,
        base_stats: &BaseStat,
    ) -> anyhow::Result<()> {
        target.class_id = self.class_id as i8;
        let p = self.get_random_loc()?;
        //todo custom starting loc
        target.x = p.x;
        target.y = p.y;
        target.z = p.z;
        target.base_class_id = self.class_id as i8;
        target.access_level = 0;
        target.race_id = self.class_id.get_class().race as i8;
        let base_max_hp =
            self.get_base_max_parameter(target.level as u8, &CreatureParameter::HP)?;
        let base_max_mp =
            self.get_base_max_parameter(target.level as u8, &CreatureParameter::MP)?;
        let base_max_cp =
            self.get_base_max_parameter(target.level as u8, &CreatureParameter::CP)?;
        let base_con = base_stats.con_bonus(u8::try_from(self.static_data.base_con)?)?;
        let base_men = base_stats.con_bonus(u8::try_from(self.static_data.base_men)?)?;
        target.max_hp = f64::from(base_max_hp) * base_con;
        target.max_mp = f64::from(base_max_mp) * base_men;
        target.max_cp = f64::from(base_max_cp) * base_con;
        target.cur_hp = target.max_hp;
        target.cur_mp = target.max_mp;
        target.cur_cp = target.max_cp;
        target.variables = json!({
            CharVariables::VisualHairStyleId.as_key(): target.hair_style,
            CharVariables::VisualHairColorId.as_key(): target.hair_color,
            CharVariables::VisualFaceId.as_key(): target.face
        });
        //todo skill tree
        //todo shortcut panel initialization
        //todo initial items
        //todo vitality
        //todo starting level
        //todo starting adena
        Ok(())
    }
    /// # Errors
    /// - when lvl is higher than we have data for it in th template.
    pub fn get_base_max_parameter(
        &self,
        lvl: u8,
        parameter: &CreatureParameter,
    ) -> anyhow::Result<f32> {
        if let Some(val) = self.lvl_up_gain_data.get(&lvl) {
            return match parameter {
                CreatureParameter::CP => Ok(val.cp),
                CreatureParameter::HP => Ok(val.hp),
                CreatureParameter::MP => Ok(val.mp),
            };
        }
        bail!("No max {:?} found for lvl {lvl}", parameter);
    }
    /// # Errors
    /// - when no creation points are specified in the template
    pub fn get_random_loc(&self) -> anyhow::Result<&Point> {
        let mut rng = thread_rng();
        if let Some(random_item) = self.static_data.creation_points.choose(&mut rng) {
            Ok(random_item)
        } else {
            bail!(
                "Creation points are not specified in template {:?}!",
                self.class_id
            );
        }
    }
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
    #[must_use]
    pub fn total(&self) -> i32 {
        self.chest + self.legs + self.head + self.feet + self.gloves + self.underwear + self.cloak
    }
}

impl BaseMDef {
    #[must_use]
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
#[cfg(test)]
mod test {
    use l2_core::config::traits::ConfigDirLoader;
    use crate::data::char_template::ClassTemplates;

    #[test]
    fn test(){
        let temps = ClassTemplates::load();
        assert_eq!(9, temps.get_available_templates_for_registration().len());
    }
}