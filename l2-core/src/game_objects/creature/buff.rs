use crate::game_objects::stats::calculator::Modifier;
use crate::game_objects::stats::stat_enum::Stat;
use chrono::{DateTime, Utc};

/// A continuous effect (buff or debuff) currently applied to a creature.
#[derive(Debug, Clone)]
pub struct AppliedBuff {
    pub skill_id: i32,
    pub skill_level: i32,
    pub caster_id: i32,
    /// Buff slot type from skill data (e.g. `PA_UP`); same type + lower/equal
    /// abnormal level gets replaced on re-apply.
    pub abnormal_type: Option<String>,
    pub abnormal_level: i32,
    pub end_time: DateTime<Utc>,
    /// Stat modifiers applied while the buff is active.
    pub mods: Vec<(Stat, Modifier)>,
}

impl AppliedBuff {
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.end_time <= Utc::now()
    }

    /// Remaining duration in seconds (0 when expired).
    #[must_use]
    pub fn remaining_secs(&self) -> i32 {
        let secs = self
            .end_time
            .signed_duration_since(Utc::now())
            .num_seconds();
        i32::try_from(secs.max(0)).unwrap_or(i32::MAX)
    }
}

#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct BuffInfo;
