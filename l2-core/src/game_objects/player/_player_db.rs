use crate::data::char_template::ClassTemplates;
use crate::game_objects::player::Player;
use entities::dao::item::LocType;
use entities::entities::character;
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
        let players = characters
            .into_iter()
            .map(|(ch, items, clan)| {
                let template = templates.try_get_template(ch.class_id)?;
                let mut p = Player::new(ch, items, template.clone());
                p.clan = clan;
                Ok(p)
            })
            .collect::<anyhow::Result<Vec<Player>>>()?;
        Ok(players)
    }
}
