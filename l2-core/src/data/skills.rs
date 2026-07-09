use crate as l2_core;
use crate::config::traits::LoadFileHandler;
use macro_common::config_dir;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer};
use serde_with::{DisplayFromStr, serde_as};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use tracing::info;

/// A single `<value level="N">..</value>` / `<value fromLevel="A" toLevel="B">..</value>` entry.
#[derive(Debug, Clone)]
pub struct PerLevelValue<T> {
    pub level: Option<u8>,
    pub from_level: Option<u8>,
    pub to_level: Option<u8>,
    pub text: Option<T>,
}

/// A skill parameter that is either a single value (`$text`) or a per-level table (`value`).
#[derive(Debug, Clone)]
pub struct ValueWrapper<T> {
    pub text: Option<T>,
    pub per_level: Vec<PerLevelValue<T>>,
}

impl<T> Default for ValueWrapper<T> {
    fn default() -> Self {
        Self {
            text: None,
            per_level: Vec::new(),
        }
    }
}

impl<T> ValueWrapper<T> {
    /// Resolves the value for the given skill level: exact `@level` match first,
    /// then `@fromLevel..=@toLevel` ranges, then the level-independent `$text`.
    pub fn get(&self, level: u8) -> Option<&T> {
        if let Some(entry) = self.per_level.iter().find(|e| e.level == Some(level)) {
            return entry.text.as_ref();
        }
        if let Some(entry) = self.per_level.iter().find(|e| {
            e.level.is_none()
                && level >= e.from_level.unwrap_or(1)
                && level <= e.to_level.unwrap_or(u8::MAX)
        }) {
            return entry.text.as_ref();
        }
        self.text.as_ref()
    }
}

fn parse_scalar<T, E>(value: &serde_json::Value) -> Result<Option<T>, E>
where
    T: FromStr,
    T::Err: Display,
    E: DeError,
{
    let as_string = match value {
        serde_json::Value::Null => return Ok(None),
        serde_json::Value::String(s) => {
            if s.is_empty() || s == "null" {
                return Ok(None);
            }
            s.clone()
        }
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        other => {
            return Err(E::custom(format!(
                "unexpected value type for a skill parameter: {other:?}"
            )));
        }
    };
    // Some entries contain enchant (sub-level) formulas like `{base + base / 100 * subIndex}`
    // instead of plain values — tolerate them as absent values.
    Ok(as_string.parse().ok())
}

impl<'de, T> Deserialize<'de> for ValueWrapper<T>
where
    T: FromStr,
    T::Err: Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut raw = serde_json::Value::deserialize(deserializer)?;
        // Duplicated XML elements are converted to arrays; use the first occurrence.
        if let serde_json::Value::Array(items) = raw {
            raw = items.into_iter().next().unwrap_or(serde_json::Value::Null);
        }
        match &raw {
            serde_json::Value::Object(map) => {
                let text = match map.get("$text") {
                    Some(v) => parse_scalar(v)?,
                    None => None,
                };
                let mut per_level = Vec::new();
                if let Some(serde_json::Value::Array(entries)) = map.get("value") {
                    for entry in entries {
                        let serde_json::Value::Object(e) = entry else {
                            return Err(D::Error::custom("per-level value must be a mapping"));
                        };
                        let get_u8 = |key: &str| -> Result<Option<u8>, D::Error> {
                            match e.get(key) {
                                Some(v) => parse_scalar(v),
                                None => Ok(None),
                            }
                        };
                        per_level.push(PerLevelValue {
                            level: get_u8("@level")?,
                            from_level: get_u8("@fromLevel")?,
                            to_level: get_u8("@toLevel")?,
                            text: match e.get("$text") {
                                Some(v) => parse_scalar(v)?,
                                None => None,
                            },
                        });
                    }
                }
                Ok(Self { text, per_level })
            }
            // Plain scalar, e.g. `targetType: SELF`.
            _ => Ok(Self {
                text: parse_scalar(&raw)?,
                per_level: Vec::new(),
            }),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(
    Debug, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Hash, strum_macros::EnumString,
)]
pub enum TargetType {
    /** Advance Head Quarters (Outposts). */
    ADVANCE_BASE,
    /** Enemies in high terrain or protected by castle walls and doors. */
    ARTILLERY,
    /** Doors or treasure chests. */
    DOOR_TREASURE,
    /** Any enemies (included allies). */
    ENEMY,
    /** Friendly. */
    ENEMY_NOT,
    /** Only enemies (not included allies). */
    ENEMY_ONLY,
    /** Fortress's Flagpole. */
    FORTRESS_FLAGPOLE,
    /** Ground. */
    GROUND,
    /** Holy Artifacts from sieges. */
    HOLYTHING,
    /** Items. */
    ITEM,
    /** Nothing. */
    NONE,
    /** NPC corpses. */
    NPC_BODY,
    /** Others, except caster. */
    OTHERS,
    /** Player corpses. */
    PC_BODY,
    /** Self. */
    SELF,
    /** Servitor or pet. */
    SUMMON,
    /** Anything targetable. */
    TARGET,
    /** Wyverns. */
    WYVERN_TARGET,
    /** Mentee's Mentor. */
    MY_MENTOR,
    /** Me or my party (if any). Seen in aura skills. */
    MY_PARTY,
    /** Pet's owner. */
    OWNER_PET,
}

impl TargetType {
    #[must_use]
    pub fn is_range_ignored(&self) -> bool {
        self == &TargetType::SELF || self == &TargetType::MY_PARTY
    }
}

#[allow(non_camel_case_types)]
#[derive(
    Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash, strum_macros::EnumString, Default,
)]
pub enum AffectScope {
    BALAKAS_SCOPE,
    DEAD_PLEDGE,
    DEAD_UNION,
    FAN,
    FAN_PB,
    NONE,
    PARTY,
    DEAD_PARTY,
    PARTY_PLEDGE,
    DEAD_PARTY_PLEDGE,
    PLEDGE,
    POINT_BLANK,
    RANGE,
    RANGE_SORT_BY_HP,
    RING_RANGE,
    #[default]
    SINGLE,
    SQUARE,
    SQUARE_PB,
    STATIC_OBJECT_SCOPE,
    SUMMON_EXCEPT_MASTER,
    WYVERN_SCOPE,
}

impl AffectScope {
    /// Area scopes affect multiple creatures around an origin point.
    #[must_use]
    pub fn is_area(&self) -> bool {
        !matches!(self, Self::SINGLE | Self::NONE)
    }
}

/// One `<effect name="...">` entry of a skill: the effect handler name plus its
/// most common parameters. `power` is used by damage/heal effects, `amount`/`mode`
/// by stat (buff/debuff) effects.
#[derive(Debug, Deserialize, Clone)]
pub struct SkillEffect {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(default)]
    pub power: Option<ValueWrapper<f64>>,
    #[serde(default)]
    pub amount: Option<ValueWrapper<f64>>,
    /// `DIFF` (flat add) or `PER` (percent) for stat effects.
    #[serde(default)]
    pub mode: Option<ValueWrapper<String>>,
    #[serde(rename = "criticalChance", default)]
    pub critical_chance: Option<ValueWrapper<f64>>,
}

impl SkillEffect {
    #[must_use]
    pub fn power(&self, level: u8) -> f64 {
        self.power
            .as_ref()
            .and_then(|p| p.get(level))
            .copied()
            .unwrap_or(0.0)
    }
    #[must_use]
    pub fn amount(&self, level: u8) -> f64 {
        self.amount
            .as_ref()
            .and_then(|p| p.get(level))
            .copied()
            .unwrap_or(0.0)
    }
    /// `true` when the stat effect is percent-based (`mode: PER`).
    #[must_use]
    pub fn is_percent(&self, level: u8) -> bool {
        self.mode
            .as_ref()
            .and_then(|m| m.get(level))
            .is_some_and(|m| m == "PER")
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SkillEffects {
    #[serde(rename = "effect", default)]
    pub effects: Vec<SkillEffect>,
}

#[serde_as]
#[derive(Debug, Deserialize, Clone)]
pub struct Skill {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "@toLevel")]
    pub to_level: u8,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "@fromLevel")]
    #[serde(default)]
    pub from_level: Option<u8>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "@displayId")]
    #[serde(default)]
    pub display_id: Option<i32>,

    pub icon: Option<ValueWrapper<String>>,
    #[serde(rename = "operateType")]
    pub operate_type: Option<ValueWrapper<String>>,
    #[serde(rename = "targetType")]
    pub target_type: Option<ValueWrapper<TargetType>>,
    #[serde(rename = "affectScope")]
    pub affect_scope: Option<ValueWrapper<AffectScope>>,
    #[serde(rename = "affectObject")]
    pub affect_object: Option<ValueWrapper<String>>,
    #[serde(rename = "affectRange")]
    pub affect_range: Option<ValueWrapper<i32>>,
    #[serde(rename = "affectLimit")]
    pub affect_limit: Option<ValueWrapper<String>>,
    #[serde(rename = "castRange")]
    pub cast_range: Option<ValueWrapper<i32>>,
    #[serde(rename = "effectRange")]
    pub effect_range: Option<ValueWrapper<i32>>,
    #[serde(rename = "effectPoint")]
    pub effect_point: Option<ValueWrapper<i32>>,
    #[serde(rename = "hitTime")]
    pub hit_time: Option<ValueWrapper<i32>>,
    #[serde(rename = "coolTime")]
    pub cool_time: Option<ValueWrapper<i32>>,
    #[serde(rename = "reuseDelay")]
    pub reuse_delay: Option<ValueWrapper<i32>>,
    #[serde(rename = "mpConsume")]
    pub mp_consume: Option<ValueWrapper<i32>>,
    #[serde(rename = "mpInitialConsume")]
    pub mp_initial_consume: Option<ValueWrapper<i32>>,
    #[serde(rename = "hpConsume")]
    pub hp_consume: Option<ValueWrapper<i32>>,
    #[serde(rename = "isMagic")]
    pub is_magic: Option<ValueWrapper<i32>>,
    #[serde(rename = "isDebuff")]
    pub is_debuff: Option<ValueWrapper<bool>>,
    #[serde(rename = "magicLevel")]
    pub magic_level: Option<ValueWrapper<i32>>,
    #[serde(rename = "magicCriticalRate")]
    pub magic_critical_rate: Option<ValueWrapper<i32>>,
    #[serde(rename = "hitCancelTime")]
    pub hit_cancel_time: Option<ValueWrapper<f32>>,
    #[serde(rename = "attributeType")]
    pub attribute_type: Option<ValueWrapper<String>>,
    #[serde(rename = "attributeValue")]
    pub attribute_value: Option<ValueWrapper<i32>>,

    #[serde(rename = "abnormalLevel")]
    pub abnormal_level: Option<ValueWrapper<i32>>,
    #[serde(rename = "abnormalTime")]
    pub abnormal_time: Option<ValueWrapper<i32>>,
    #[serde(rename = "abnormalType")]
    pub abnormal_type: Option<ValueWrapper<String>>,
    #[serde(rename = "abnormalVisualEffect")]
    pub abnormal_visual_effect: Option<ValueWrapper<String>>,

    #[serde(rename = "basicProperty")]
    pub basic_property: Option<ValueWrapper<String>>,
    #[serde(rename = "staticReuse")]
    pub static_reuse: Option<ValueWrapper<bool>>,
    #[serde(rename = "reuseDelayGroup")]
    pub reuse_delay_group: Option<ValueWrapper<i32>>,

    pub effects: Option<SkillEffects>,
}

fn resolve<T: Copy>(field: Option<&ValueWrapper<T>>, level: u8, default: T) -> T {
    field.and_then(|w| w.get(level)).copied().unwrap_or(default)
}

impl Skill {
    #[must_use]
    pub fn target_type_at(&self, level: u8) -> TargetType {
        resolve(self.target_type.as_ref(), level, TargetType::SELF)
    }
    #[must_use]
    pub fn affect_scope_at(&self, level: u8) -> AffectScope {
        resolve(self.affect_scope.as_ref(), level, AffectScope::SINGLE)
    }
    #[must_use]
    pub fn cast_range_at(&self, level: u8) -> i32 {
        resolve(self.cast_range.as_ref(), level, -1)
    }
    #[must_use]
    pub fn effect_range_at(&self, level: u8) -> i32 {
        resolve(self.effect_range.as_ref(), level, -1)
    }
    #[must_use]
    pub fn affect_range_at(&self, level: u8) -> i32 {
        resolve(self.affect_range.as_ref(), level, 0)
    }
    /// Parses "min-max" affect limit; returns the max amount of affected targets
    /// or `i32::MAX` when unlimited.
    #[must_use]
    pub fn affect_limit_at(&self, level: u8) -> i32 {
        self.affect_limit
            .as_ref()
            .and_then(|w| w.get(level))
            .and_then(|s| {
                let mut parts = s.split('-');
                let min: i32 = parts.next()?.trim().parse().ok()?;
                let rand_part: i32 = parts
                    .next()
                    .and_then(|p| p.trim().parse().ok())
                    .unwrap_or(0);
                Some(min + rand_part)
            })
            .filter(|&limit| limit > 0)
            .unwrap_or(i32::MAX)
    }
    #[must_use]
    pub fn effect_point_at(&self, level: u8) -> i32 {
        resolve(self.effect_point.as_ref(), level, 0)
    }
    /// A skill is "bad" (offensive) when its effect point is negative.
    #[must_use]
    pub fn is_bad(&self, level: u8) -> bool {
        self.effect_point_at(level) < 0
    }
    #[must_use]
    pub fn is_magic(&self) -> bool {
        resolve(self.is_magic.as_ref(), 1, 0) != 0
    }
    #[must_use]
    pub fn is_debuff(&self) -> bool {
        resolve(self.is_debuff.as_ref(), 1, false)
    }
    #[must_use]
    pub fn hit_time_at(&self, level: u8) -> i32 {
        resolve(self.hit_time.as_ref(), level, 0)
    }
    #[must_use]
    pub fn cool_time_at(&self, level: u8) -> i32 {
        resolve(self.cool_time.as_ref(), level, 0)
    }
    #[must_use]
    pub fn reuse_delay_at(&self, level: u8) -> i32 {
        resolve(self.reuse_delay.as_ref(), level, 0)
    }
    #[must_use]
    pub fn reuse_delay_group_at(&self, level: u8) -> i32 {
        resolve(self.reuse_delay_group.as_ref(), level, -1)
    }
    #[must_use]
    pub fn mp_consume_at(&self, level: u8) -> i32 {
        resolve(self.mp_consume.as_ref(), level, 0)
    }
    #[must_use]
    pub fn mp_initial_consume_at(&self, level: u8) -> i32 {
        resolve(self.mp_initial_consume.as_ref(), level, 0)
    }
    #[must_use]
    pub fn hp_consume_at(&self, level: u8) -> i32 {
        resolve(self.hp_consume.as_ref(), level, 0)
    }
    /// Buff/debuff duration in seconds.
    #[must_use]
    pub fn abnormal_time_at(&self, level: u8) -> i32 {
        resolve(self.abnormal_time.as_ref(), level, 0)
    }
    #[must_use]
    pub fn magic_critical_rate_at(&self, level: u8) -> i32 {
        resolve(self.magic_critical_rate.as_ref(), level, 0)
    }
    #[must_use]
    pub fn effects(&self) -> &[SkillEffect] {
        self.effects.as_ref().map_or(&[], |e| &e.effects)
    }
    /// `A2`/`A3`/`T`/dances have continuous effects that stay for `abnormalTime`.
    #[must_use]
    pub fn is_continuous(&self) -> bool {
        self.operate_type
            .as_ref()
            .and_then(|w| w.text.as_ref())
            .is_some_and(|op| {
                matches!(op.as_str(), "A2" | "A3" | "A4" | "A5" | "DA1" | "DA2" | "T")
            })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SkillList {
    pub skill: Vec<Skill>,
}

#[derive(Debug, Clone, Default)]
#[config_dir(path = "config/data/stats/skills", post_load)]
pub struct SkillsData {
    pub skills: HashMap<u32, Vec<Skill>>,
}

impl l2_core::config::traits::Loadable for SkillsData {
    fn post_load(&self) {
        info!("Loaded {} skill groups.", self.skills.len());
    }
}

impl SkillsData {
    #[must_use]
    pub fn get_skill(&self, skill_id: u32, level: u8) -> Option<&Skill> {
        self.skills.get(&skill_id).and_then(|levels| {
            levels.iter().find(|s| {
                let to_level = s.to_level;
                let from_level = s.from_level.unwrap_or(1);
                level >= from_level && level <= to_level
            })
        })
    }
}

impl LoadFileHandler for SkillsData {
    type TargetConfigType = SkillList;

    fn for_each(&mut self, item: Self::TargetConfigType) {
        for skill in item.skill {
            self.skills.entry(skill.id).or_default().push(skill);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_skill_with_quoted_numbers() {
        let yaml = r#"
'@id': '26900'
'@toLevel': '1'
'@name': Test Skill
'@fromLevel': '1'
'@displayId': '26900'
"#;
        let skill: Skill = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(skill.id, 26900);
        assert_eq!(skill.to_level, 1);
        assert_eq!(skill.from_level, Some(1));
        assert_eq!(skill.display_id, Some(26900));
    }

    #[test]
    fn test_deserialize_skill_with_float() {
        let yaml = r#"
'@id': '9453'
'@toLevel': '1'
'@name': Test Skill
hitCancelTime:
  $text: '0.2'
"#;
        let skill: Skill = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(skill.hit_cancel_time.unwrap().text, Some(0.2));
    }

    #[test]
    fn test_deserialize_per_level_values() {
        let yaml = r#"
'@id': '1177'
'@toLevel': '5'
'@name': Wind Strike
castRange:
  $text: '600'
effectPoint:
  value:
  - '@level': '1'
    $text: '-92'
  - '@level': '2'
    $text: '-106'
targetType:
  value:
  - '@level': '1'
    $text: ENEMY_ONLY
  - '@level': '2'
    $text: ENEMY
affectScope:
  $text: SINGLE
effects:
  effect:
  - '@name': MagicalAttack
    power:
      value:
      - '@fromLevel': '1'
        '@toLevel': '2'
        $text: '12'
      - '@fromLevel': '3'
        '@toLevel': '5'
        $text: '15'
"#;
        let skill: Skill = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(skill.cast_range_at(1), 600);
        assert_eq!(skill.effect_point_at(2), -106);
        assert!(skill.is_bad(1));
        assert_eq!(skill.target_type_at(1), TargetType::ENEMY_ONLY);
        assert_eq!(skill.target_type_at(2), TargetType::ENEMY);
        assert_eq!(skill.affect_scope_at(1), AffectScope::SINGLE);
        let effects = skill.effects();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].name, "MagicalAttack");
        assert_eq!(effects[0].power(2), 12.0);
        assert_eq!(effects[0].power(4), 15.0);
    }

    #[test]
    fn test_deserialize_buff_effect() {
        let yaml = r#"
'@id': '1068'
'@toLevel': '3'
'@name': Might
abnormalTime:
  $text: '1200'
abnormalType:
  $text: PA_UP
operateType:
  $text: A2
targetType:
  $text: TARGET
effectPoint:
  value:
  - '@level': '1'
    $text: '121'
effects:
  effect:
  - '@name': PAtk
    amount:
      value:
      - '@level': '1'
        $text: '8'
      - '@level': '2'
        $text: '12'
    mode:
      $text: PER
"#;
        let skill: Skill = serde_yaml::from_str(yaml).unwrap();
        assert!(!skill.is_bad(1));
        assert!(skill.is_continuous());
        assert_eq!(skill.abnormal_time_at(1), 1200);
        let effect = &skill.effects()[0];
        assert_eq!(effect.amount(2), 12.0);
        assert!(effect.is_percent(1));
    }
}

#[cfg(test)]
mod load_tests {
    use super::*;
    use crate::config::traits::ConfigDirLoader;

    #[test]
    fn test_load_all_skill_files() {
        // Only runs when the full config is present (repo root).
        if std::env::var("L2_CONFIG").is_err() {
            return;
        }
        let data = SkillsData::load();
        assert!(
            data.skills.len() > 10_000,
            "expected all skill groups loaded"
        );
        let wind_strike = data.get_skill(1177, 1).expect("Wind Strike missing");
        assert_eq!(wind_strike.cast_range_at(1), 600);
        assert_eq!(wind_strike.target_type_at(1), TargetType::ENEMY_ONLY);
        assert_eq!(wind_strike.target_type_at(2), TargetType::ENEMY);
        assert!(wind_strike.is_bad(1));
        assert_eq!(wind_strike.effects()[0].name, "MagicalAttack");
        assert!(wind_strike.effects()[0].power(1) > 0.0);

        let might = data.get_skill(1068, 1).expect("Might missing");
        assert!(might.is_continuous());
        assert!(!might.is_bad(1));
        assert_eq!(might.abnormal_time_at(1), 1200);
        assert_eq!(might.effects()[0].name, "PAtk");
        assert_eq!(might.effects()[0].amount(1), 8.0);
        assert!(might.effects()[0].is_percent(1));

        let heal = data.get_skill(1011, 1).expect("Heal missing");
        assert_eq!(heal.effects()[0].name, "Heal");
    }
}
