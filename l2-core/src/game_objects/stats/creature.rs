use crate::game_objects::stats::calculator::StatCalculator;
use crate::game_objects::stats::stat_enum::Stat;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CreatureStats {
    pub calculator: StatCalculator,
    pub current_hp: f64,
    pub current_mp: f64,
    pub current_cp: f64,
    pub cached_stats: HashMap<Stat, f64>,
}

impl CreatureStats {
    pub fn new() -> Self {
        Self {
            calculator: StatCalculator::new(),
            current_hp: 0.0,
            current_mp: 0.0,
            current_cp: 0.0,
            cached_stats: HashMap::new(),
        }
    }

    pub fn update_cache(&mut self) {
        self.cached_stats = self.calculator.calculate_all();
    }

    pub fn get_stat(&self, stat: Stat) -> f64 {
        self.cached_stats.get(&stat).cloned().unwrap_or_else(|| {
            // If not in cache, calculate it individually (might be slow if done often)
            self.calculator.calculate_stat(stat, &self.cached_stats)
        })
    }
}