use chrono::{DateTime, Utc};
use entities::entities::skill;
use entities::DBPool;

#[derive(Debug, Clone)]
pub struct SkillReuse {
    pub skill_id: i32,
    pub skill_level: i32,
    pub reuse_delay: i64,
    pub end_time: DateTime<Utc>,
    pub shared_reuse_group: i32,
}

impl SkillReuse {
    pub fn has_not_passed(&self) -> bool {
        self.end_time > Utc::now()
    }

    pub fn get_remaining(&self) -> i64 {
        let remaining = self.end_time.signed_duration_since(Utc::now()).num_milliseconds();
        if remaining > 0 {
            remaining
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub model: skill::Model,
    pub passive: bool,
    pub disabled: bool,
    pub reuse_delay_group: i32,
    pub can_enchant: bool,
}
impl Skill {
    /// # Errors
    /// - when DB is not accessible
    pub async fn load_for_char(pool: &DBPool, char_id: i32) -> anyhow::Result<Vec<Skill>> {
        let skills = skill::Model::char_skills(pool, char_id).await?;
        Ok(skills.into_iter().map(Skill::from_model).collect())
    }
    #[must_use]
    pub fn from_model(model: skill::Model) -> Self {
        //todo: implement me
        Self {
            model,
            passive: false,
            disabled: false,
            can_enchant: false,
            reuse_delay_group: -1,
        }
    }

    pub fn from_data(_skill_data: &crate::data::skills::Skill) -> Self {
        // This is a bit of a hack since we don't have a full skill::Model here
        // But for many logic parts it might be enough or we need to merge them
        unimplemented!("Conversion from SkillsData Skill to Creature Skill is not fully implemented")
    }
}
