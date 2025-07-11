use crate as l2_core;
use crate::config::traits::{LoadFileHandler, Loadable};
use crate::data::classes::mapping::Class;
use crate::game_objects::race::Race;
use macro_common::config_dir;
use serde::{Deserialize, Deserializer};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum SocialClass {
    Vagabond,
    Vassal,
    Apprentice,
    Heir,
    Knight,
    Elder,
    Baron,
    Viscount,
    Count,
    Marquis,
    Duke,
    GrandDuke,
    DistinguishedKing,
    Emperor,
}
#[derive(Clone, Debug, Deserialize, Default)]
pub struct RemoveTreeSkill {
    skill_id: u32,
    only_replace_by_learn: bool,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct PreRequisiteSkill {
    skill_id: u32,
    lvl: u32,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct TreeSkillItem {
    id: u32,
    count: u32,
}
#[derive(Clone, Debug, Deserialize, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct TreeSkill {
    skill_id: u32,
    skill_level: u8,
    skill_name: String,
    #[serde(default)]
    auto_get: bool,
    #[serde(default)]
    get_dual_class_level: u8,
    #[serde(default)]
    learned_by_f_s: bool,
    #[serde(default)]
    learned_by_npc: bool,
    #[serde(default)]
    level_up_sp: u64,
    #[serde(default)]
    residence_skill: bool,
    #[serde(default)]
    get_level: u8,
    #[serde(default)]
    social_class: Option<SocialClass>,
    #[serde(default)]
    remove_skill: Vec<RemoveTreeSkill>,
    #[serde(default)]
    residence_ids: HashSet<u8>,
    #[serde(default)]
    races: HashSet<Race>,
    #[serde(default)]
    pre_requisite_skills: Vec<PreRequisiteSkill>,
    #[serde(default)]
    items: Vec<TreeSkillItem>,
    #[serde(default)]
    tree_id: u32,
    #[serde(default)]
    row: u32,
    #[serde(default)]
    column: u32,
    #[serde(default)]
    points_required: u32,
}

impl TreeSkill {
    #[must_use]
    pub fn hash(&self) -> u64 {
        (u64::from(self.skill_id) * 4_294_967_296_u64) + u64::from(self.skill_level)
    }
}

#[derive(Clone, Debug, Default)]
#[config_dir(path = "config/data/skill_trees", post_load)]
pub struct SkillTreesData {
    class_skill_trees: HashMap<Class, Arc<SkillTree>>,
    common_skill_trees: Arc<SkillTree>,
}

#[derive(Debug, Deserialize, Clone, Default, Eq, PartialEq, Hash, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum SkillTreeType {
    AbilitySkillTree,
    AlchemySkillTree,
    AwakeningSaveSkillTree,
    #[default]
    ClassSkillTree,
    CollectSkillTree,
    DualClassSkillTree,
    FishingSkillTree,
    GameMasterAuraSkillTree,
    GameMasterSkillTree,
    HeroSkillTree,
    NobleSkillTree,
    PledgeSkillTree,
    RaceSkillTree,
    RevelationSkillTree,
    SubClassSkillTree,
    SubClassChangeSkillTree,
    SubPledgeSkillTree,
    TransferSkillTree,
    TransformSkillTree,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Subtype {
    BaseClass,
    DualClass,
}
#[derive(Clone, Debug, Deserialize, Default)]
pub struct SkillTree {
    pub class_id: Option<Class>,
    pub parent_class_id: Option<Class>,
    pub skill_tree_type: SkillTreeType,
    #[serde(deserialize_with = "deserialize_skills")]
    pub skills: HashMap<u64, TreeSkill>,
    pub race: Option<Race>,
    pub subtype: Option<String>,
}
fn deserialize_skills<'de, D>(deserializer: D) -> Result<HashMap<u64, TreeSkill>, D::Error>
where
    D: Deserializer<'de>,
{
    let vec = Vec::<TreeSkill>::deserialize(deserializer)?;
    let mut map = HashMap::with_capacity(vec.len());
    for skill in vec {
        map.insert(skill.hash(), skill);
    }
    Ok(map)
}

impl Loadable for SkillTreesData {
    fn post_load(&self) {
        info!("Loaded class skill trees for {} classes.", self.class_skill_trees.len());
        info!("Loaded {} skills for common tree.", self.common_skill_trees.skills.len());
    }
}

impl LoadFileHandler for SkillTreesData {
    type TargetConfigType = Arc<SkillTree>;
    fn for_each(&mut self, item: Self::TargetConfigType) {
        match (item.skill_tree_type, item.class_id) {
            (SkillTreeType::ClassSkillTree, Some(c_id)) => {
                self.class_skill_trees.insert(c_id, item);
            }
            (SkillTreeType::ClassSkillTree, _) => {
                self.common_skill_trees = item;
            }
            (_, _) => {}
        }
    }
}
