use crate as l2_core;
use crate::config::traits::LoadFileHandler;
use macro_common::config_dir;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct ValueWrapper<T> {
    #[serde(rename = "$text")]
    pub text: Option<T>,
    #[serde(rename = "$value")]
    pub value: Option<T>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Skill {
    #[serde(rename = "@id")]
    pub id: u32,
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@toLevel")]
    pub to_level: u8,
    #[serde(rename = "@fromLevel")]
    pub from_level: Option<u8>,
    #[serde(rename = "@displayId")]
    pub display_id: Option<u32>,

    pub icon: Option<ValueWrapper<String>>,
    pub operateType: Option<ValueWrapper<String>>,
    pub targetType: Option<ValueWrapper<String>>,
    pub affectScope: Option<ValueWrapper<String>>,
    pub castRange: Option<ValueWrapper<i32>>,
    pub effectRange: Option<ValueWrapper<i32>>,
    pub hitTime: Option<ValueWrapper<u32>>,
    pub coolTime: Option<ValueWrapper<u32>>,
    pub reuseDelay: Option<ValueWrapper<u32>>,
    // Add more fields as needed based on common usage or requirements
}

#[derive(Debug, Deserialize, Clone)]
pub struct SkillList {
    pub skill: Vec<Skill>,
}

#[derive(Debug, Clone, Default)]
#[config_dir(path = "config/data/stats/skills")]
pub struct SkillsData {
    pub skills: HashMap<u32, Vec<Skill>>,
}

impl SkillsData {
    pub fn post_load(&self) {
        info!("Loaded {} skill groups.", self.skills.len());
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
