use crate as l2_core;
use crate::config::traits::{LoadFileHandler, Loadable};
use crate::data::classes::mapping::Class;
use crate::game_objects::player::Player;
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

    #[must_use]
    pub fn skill_id(&self) -> u32 {
        self.skill_id
    }

    #[must_use]
    pub fn skill_level(&self) -> u8 {
        self.skill_level
    }

    #[must_use]
    pub fn get_level(&self) -> u8 {
        self.get_level
    }

    #[must_use]
    pub fn level_up_sp(&self) -> u64 {
        self.level_up_sp
    }

    #[must_use]
    pub fn items(&self) -> &[TreeSkillItem] {
        &self.items
    }

    #[must_use]
    pub fn remove_skills(&self) -> &[RemoveTreeSkill] {
        &self.remove_skill
    }
}

impl TreeSkillItem {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn count(&self) -> u32 {
        self.count
    }
}

impl RemoveTreeSkill {
    pub fn skill_id(&self) -> u32 {
        self.skill_id
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

impl SkillTreesData {
    pub fn post_load(&self) {
        info!(
            "Loaded class skill trees for {} classes.",
            self.class_skill_trees.len()
        );
        info!(
            "Loaded {} skills for common tree.",
            self.common_skill_trees.skills.len()
        );
    }

    #[must_use]
    pub fn get_available_skills(&self, player: &Player) -> Vec<TreeSkill> {
        let mut available_skills = Vec::new();
        let Ok(class_id) = player.char_model.class_id.try_into() else {
            return available_skills;
        };
        let char_level = player.char_model.level as u8;

        // 1. Get all relevant skills for this class (including parents)
        let mut complete_tree = Vec::new();
        let mut current_class = Some(class_id);
        while let Some(c) = current_class {
            if let Some(tree) = self.class_skill_trees.get(&c) {
                complete_tree.extend(tree.skills.values());
            }
            current_class = c.get_class().parent;
        }
        complete_tree.extend(self.common_skill_trees.skills.values());

        // 2. Identify currently learnable skills
        for skill in &complete_tree {
            if self.is_learnable(player, skill, char_level) {
                available_skills.push((*skill).clone());
            }
        }

        // 3. Identify next available skills (skills for the next level-up)
        let mut min_next_level = None;
        for skill in &complete_tree {
            if !skill.auto_get && skill.get_level > char_level {
                if min_next_level.is_none() || Some(skill.get_level) < min_next_level {
                    min_next_level = Some(skill.get_level);
                }
            }
        }

        if let Some(next_lvl) = min_next_level {
            for skill in &complete_tree {
                if !skill.auto_get && skill.get_level == next_lvl {
                    // Check if this skill would be learnable if we were at that level
                    if self.is_learnable_at_level(player, skill, next_lvl) {
                        // Avoid duplicates if it's already in available_skills (shouldn't be)
                        if !available_skills.iter().any(|s| {
                            s.skill_id == skill.skill_id && s.skill_level == skill.skill_level
                        }) {
                            available_skills.push((*skill).clone());
                        }
                    }
                }
            }
        }

        available_skills
    }

    fn is_learnable(&self, player: &Player, skill: &TreeSkill, char_level: u8) -> bool {
        if skill.auto_get {
            return false;
        }

        self.is_learnable_at_level(player, skill, char_level)
    }

    fn is_learnable_at_level(&self, player: &Player, skill: &TreeSkill, char_level: u8) -> bool {
        // Enforce level constraint
        if char_level < skill.get_level {
            return false;
        }

        // Verify player's race is allowed by skill.races
        if !skill.races.is_empty() {
            if let Ok(race) = player.try_get_race() {
                if !skill.races.contains(&race) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Verify player's social class matches skill.social_class
        if let Some(social_class) = skill.social_class {
            if player.get_pledge_class() < social_class as u8 {
                return false;
            }
        }

        // Ensure residence/location requirements are satisfied
        if !skill.residence_ids.is_empty() {
            // This is a placeholder as the exact residence check depends on clan/residence implementation
            // For now, check if player has a clan and if its ID is in residence_ids (might be wrong logic for residence_ids)
            if let Some(clan) = &player.clan {
                if !skill.residence_ids.contains(&(clan.id as u8)) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Validate dual-class level or class-specific restrictions
        if skill.get_dual_class_level > 0 {
            let has_dual_class_requirement = player.sub_classes.iter().any(|s| {
                s.class_type == crate::game_objects::player::SubclassType::DualClass
                    && s.level >= skill.get_dual_class_level
            });
            if !has_dual_class_requirement {
                return false;
            }
        }

        // Check pre_requisite_skills are owned at required levels
        for pre_req in &skill.pre_requisite_skills {
            let mut found = false;
            if let Some(skills) = &player.skills {
                for player_skill in skills {
                    if player_skill.model.id == pre_req.skill_id as i32 {
                        if player_skill.model.level >= pre_req.lvl as i16 {
                            found = true;
                            break;
                        }
                    }
                }
            }
            if !found {
                return false;
            }
        }

        // Check if player already has this skill at this level or higher
        if let Some(skills) = &player.skills {
            for player_skill in skills {
                if player_skill.model.id == skill.skill_id as i32 {
                    if player_skill.model.level >= skill.skill_level as i16 {
                        return false;
                    }
                    // If player has lower level, check if this is the next level
                    return player_skill.model.level + 1 == skill.skill_level as i16;
                }
            }
        }

        // If player doesn't have the skill at all, only level 1 is learnable
        skill.skill_level == 1
    }

    #[must_use]
    pub fn get_initial_skills(&self, class_id: Class, char_level: u8) -> Vec<TreeSkill> {
        let mut initial_skills = Vec::new();

        // 1. Get skills from the class-specific tree
        if let Some(tree) = self.class_skill_trees.get(&class_id) {
            for skill in tree.skills.values() {
                if skill.auto_get && char_level >= skill.get_level {
                    initial_skills.push(skill.clone());
                }
            }
        }

        // 2. Get skills from the common tree (if applicable)
        for skill in self.common_skill_trees.skills.values() {
            if skill.auto_get && char_level >= skill.get_level {
                initial_skills.push(skill.clone());
            }
        }

        initial_skills
    }
}

impl Loadable for SkillTreesData {}

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
