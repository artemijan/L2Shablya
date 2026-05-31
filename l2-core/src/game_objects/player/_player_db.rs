use crate::data::char_template::ClassTemplates;
use crate::game_objects::creature::skill::Skill;
use crate::game_objects::player::Player;
use entities::dao::item::LocType;
use entities::entities::{character, skill};
use entities::DBPool;

impl Player {
    /// # Errors
    /// - when db connection fails
    /// - when char `template_id` doesn't match what's inside DB
    pub async fn load_account_chars_from_db(
        acc_name: &str,
        db_pool: &DBPool,
        templates: &ClassTemplates,
    ) -> anyhow::Result<Vec<Self>> {
        // Fetch user characters from the database
        let characters =
            character::Model::load_chars_with_data(db_pool, acc_name, LocType::Paperdoll).await?;
        let mut players = Vec::with_capacity(characters.len());
        for (ch, items, clan) in characters {
            let template = templates.try_get_template(ch.class_id)?;
            let skills = skill::Model::char_skills(db_pool, ch.id).await?;
            let player_skills: Vec<Skill> = skills.into_iter().map(Skill::from_model).collect();
            let mut p = Player::new(ch, items, template.clone(), Some(player_skills));
            p.clan = clan;
            players.push(p);
        }
        Ok(players)
    }
}
