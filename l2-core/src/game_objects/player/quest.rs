use crate::game_objects::player::vars::QuestVariables;
use entities::entities::quest;
use sea_orm::JsonValue;

#[derive(Debug, Clone)]
pub struct Quest {
    pub model: quest::Model,

}
impl Quest {
    
    #[must_use]
    pub fn get_id(&self) -> i32 {
        self.model.quest_id
    }
    pub fn has_state(&self, state: &str) -> bool {
        if let Some(state) = self
            .model
            .variables
            .get(QuestVariables::State.as_key())
            .and_then(JsonValue::as_str)
            && state.eq(state)
        {
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn is_started(&self) -> bool {
        self.has_state("started")
    }
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.has_state("completed")
    }

    #[must_use]
    pub fn get_condition_bit_set(&self) -> u32 {
        if self.is_started() {
            let mut val = self
                .model
                .variables
                .get(QuestVariables::Condition.as_key())
                .and_then(JsonValue::as_i64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(0);
            if (val & 0x8000_0000) != 0 {
                val &= 0x7fff_ffff;
                for i in 0..32 {
                    val >>= 1;
                    if val == 0 {
                        val = i;
                        break;
                    }
                }
            }
            return val;
        }
        0
    }
}
