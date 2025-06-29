use entities::entities::skill;
use entities::DBPool;

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
            reuse_delay_group: 0
        }
    }
}
