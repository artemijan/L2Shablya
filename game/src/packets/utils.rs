use crate::packets::enums::CharNameResponseVariant;
use entities::entities::character;
use entities::DBPool;

pub async fn validate_can_create_char(
    db_pool: &DBPool,
    char_name: &str,
) -> anyhow::Result<CharNameResponseVariant> {
    if char_name.len() > 16 || char_name.len() < 3 {
        return Ok(CharNameResponseVariant::InvalidLength);
    } else if !char_name.chars().all(char::is_alphanumeric) {
        return Ok(CharNameResponseVariant::InvalidName);
    } else if character::Model::char_exists(db_pool, char_name).await? {
        return Ok(CharNameResponseVariant::AlreadyExists);
    }
    Ok(CharNameResponseVariant::Ok)
}
