use crate as l2_core;
use crate::config::traits::LoadFileHandler;
use macro_common::config_dir;
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ValueWrapper<T> {
    #[serde(rename = "$text")]
    pub text: Option<T>,
    #[serde(rename = "$value")]
    pub value: Option<T>,
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
    pub from_level: Option<u8>,
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(rename = "@displayId")]
    pub display_id: Option<i32>,

    pub icon: Option<ValueWrapper<String>>,
    #[serde(rename = "operateType")]
    pub operate_type: Option<ValueWrapper<String>>,
    #[serde(rename = "targetType")]
    pub target_type: Option<ValueWrapper<String>>,
    #[serde(rename = "affectScope")]
    pub affect_scope: Option<ValueWrapper<String>>,
    #[serde(rename = "castRange")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub cast_range: Option<ValueWrapper<i32>>,
    #[serde(rename = "effectRange")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub effect_range: Option<ValueWrapper<i32>>,
    #[serde(rename = "hitTime")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub hit_time: Option<ValueWrapper<i32>>,
    #[serde(rename = "coolTime")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub cool_time: Option<ValueWrapper<i32>>,
    #[serde(rename = "reuseDelay")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub reuse_delay: Option<ValueWrapper<i32>>,
    #[serde(rename = "isMagic")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub is_magic: Option<ValueWrapper<i32>>,
    #[serde(rename = "magicLevel")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub magic_level: Option<ValueWrapper<i32>>,
    #[serde(rename = "magicCriticalRate")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub magic_critical_rate: Option<ValueWrapper<i32>>,
    #[serde(rename = "hitCancelTime")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub hit_cancel_time: Option<ValueWrapper<f32>>,

    #[serde(rename = "abnormalLevel")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub abnormal_level: Option<ValueWrapper<i32>>,

    #[serde(rename = "abnormalTime")]
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_wrapper_opt_from_str")]
    pub abnormal_time: Option<ValueWrapper<i32>>,

    #[serde(rename = "abnormalType")]
    pub abnormal_type: Option<ValueWrapper<String>>,

    #[serde(rename = "abnormalVisualEffect")]
    pub abnormal_visual_effect: Option<ValueWrapper<String>>,

    #[serde(rename = "basicProperty")]
    pub basic_property: Option<ValueWrapper<String>>,

    #[serde(rename = "staticReuse")]
    pub static_reuse: Option<ValueWrapper<String>>,
}

fn deserialize_wrapper_opt_from_str<'de, T, D>(
    deserializer: D,
) -> Result<Option<ValueWrapper<T>>, D::Error>
where
    T: std::str::FromStr + Deserialize<'de>,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    pub struct RawValueWrapper {
        #[serde(rename = "$text")]
        pub text: Option<serde_json::Value>,
        #[serde(rename = "$value")]
        pub value: Option<serde_json::Value>,
    }

    let raw: Option<RawValueWrapper> = Option::deserialize(deserializer)?;
    if let Some(r) = raw {
        let text = match r.text {
            Some(serde_json::Value::String(s)) => {
                if s.is_empty() || s == "null" {
                    None
                } else {
                    Some(s.parse().map_err(serde::de::Error::custom)?)
                }
            }
            Some(serde_json::Value::Number(n)) => {
                if n.is_f64() {
                    let s = n.to_string();
                    Some(s.parse().map_err(serde::de::Error::custom)?)
                } else {
                    Some(T::deserialize(serde_json::Value::Number(n)).map_err(serde::de::Error::custom)?)
                }
            }
            _ => None,
        };
        let value = match r.value {
            Some(serde_json::Value::String(s)) => {
                if s.is_empty() || s == "null" {
                    None
                } else {
                    Some(s.parse().map_err(serde::de::Error::custom)?)
                }
            }
            Some(serde_json::Value::Number(n)) => {
                let s = n.to_string();
                Some(s.parse().map_err(serde::de::Error::custom)?)
            }
            _ => None,
        };
        Ok(Some(ValueWrapper { text, value }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

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
    fn test_deserialize_skill_with_unquoted_numbers() {
        let yaml = r#"
'@id': 26900
'@toLevel': 1
'@name': Test Skill
'@fromLevel': 1
'@displayId': 26900
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
}

impl LoadFileHandler for SkillsData {
    type TargetConfigType = SkillList;

    fn for_each(&mut self, item: Self::TargetConfigType) {
        for skill in item.skill {
            self.skills.entry(skill.id).or_default().push(skill);
        }
    }
}
