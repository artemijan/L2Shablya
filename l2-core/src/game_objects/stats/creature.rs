use crate::game_objects::creature::buff::AppliedBuff;
use crate::game_objects::stats::calculator::{Modifier, StatCalculator};
use crate::game_objects::stats::stat_enum::Stat;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CreatureStats {
    pub calculator: StatCalculator,
    pub current_hp: f64,
    pub current_mp: f64,
    pub current_cp: f64,
    pub cached_stats: HashMap<Stat, f64>,
    pub active_buffs: Vec<AppliedBuff>,
}

impl Default for CreatureStats {
    fn default() -> Self {
        Self::new()
    }
}

impl CreatureStats {
    pub fn new() -> Self {
        Self {
            calculator: StatCalculator::new(),
            current_hp: 0.0,
            current_mp: 0.0,
            current_cp: 0.0,
            cached_stats: HashMap::new(),
            active_buffs: Vec::new(),
        }
    }

    /// Recalculates all stats from base values and layers active buff modifiers on top.
    pub fn update_cache(&mut self) {
        self.active_buffs.retain(|b| !b.is_expired());
        let mut stats = self.calculator.calculate_all();
        for buff in &self.active_buffs {
            for (stat, modifier) in &buff.mods {
                let current = stats
                    .get(stat)
                    .copied()
                    .unwrap_or_else(|| self.calculator.calculate_stat(*stat, &stats));
                let new_value = match modifier {
                    Modifier::Add(v) => current + v,
                    Modifier::Mul(v) => current * v,
                };
                stats.insert(*stat, new_value);
            }
        }
        self.cached_stats = stats;
    }

    /// Adds (or replaces, by `abnormal_type`/skill id) a continuous effect and
    /// refreshes the stat cache. Returns `false` when a stronger buff of the same
    /// abnormal type is already active, in which case nothing changes.
    pub fn add_buff(&mut self, buff: AppliedBuff) -> bool {
        self.active_buffs.retain(|b| !b.is_expired());
        if let Some(existing) = self.active_buffs.iter().position(|b| {
            b.skill_id == buff.skill_id
                || (buff.abnormal_type.is_some() && b.abnormal_type == buff.abnormal_type)
        }) {
            if self.active_buffs[existing].abnormal_level > buff.abnormal_level {
                return false;
            }
            self.active_buffs.remove(existing);
        }
        self.active_buffs.push(buff);
        self.update_cache();
        true
    }

    /// Removes a continuous effect by skill id; returns `true` if something was removed.
    pub fn remove_buff(&mut self, skill_id: i32) -> bool {
        let before = self.active_buffs.len();
        self.active_buffs.retain(|b| b.skill_id != skill_id);
        let removed = self.active_buffs.len() != before;
        if removed {
            self.update_cache();
        }
        removed
    }

    /// Drops expired buffs; returns `true` when the cache was refreshed.
    pub fn purge_expired_buffs(&mut self) -> bool {
        if self.active_buffs.iter().any(AppliedBuff::is_expired) {
            self.update_cache();
            true
        } else {
            false
        }
    }

    pub fn get_stat(&self, stat: Stat) -> f64 {
        self.cached_stats.get(&stat).cloned().unwrap_or_else(|| {
            // If not in cache, calculate it individually (might be slow if done often)
            self.calculator.calculate_stat(stat, &self.cached_stats)
        })
    }
}
