use std::fmt;
use crate::game_objects::stats::stat_enum::Stat;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Modifier {
    Add(f64),
    Mul(f64),
}

pub trait StatFinalizer: Send + Sync {
    fn finalize(&self, base: f64, add: f64, mul: f64, stats: &HashMap<Stat, f64>) -> f64;
}

pub struct DefaultFinalizer;

impl StatFinalizer for DefaultFinalizer {
    fn finalize(&self, base: f64, add: f64, mul: f64, _stats: &HashMap<Stat, f64>) -> f64 {
        (base + add) * mul
    }
}

pub struct PAtkFinalizer;
impl StatFinalizer for PAtkFinalizer {
    fn finalize(&self, base: f64, add: f64, mul: f64, stats: &HashMap<Stat, f64>) -> f64 {
        // Example: (Base + Add) * Mul * STR_Bonus
        let str_val = stats.get(&Stat::Str).cloned().unwrap_or(40.0);
        let str_bonus = 1.0 + (str_val - 40.0) * 0.05; // Simplified bonus logic
        (base + add) * mul * str_bonus
    }
}

pub struct StatCalculator {
    pub base_values: HashMap<Stat, f64>,
    pub modifiers: HashMap<Stat, Vec<Modifier>>,
    pub finalizers: HashMap<Stat, Box<dyn StatFinalizer>>,
}

impl fmt::Debug for StatCalculator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StatCalculator")
            .field("base_values", &self.base_values)
            .field("modifiers", &self.modifiers)
            .finish()
    }
}

impl Clone for StatCalculator {
    fn clone(&self) -> Self {
        Self {
            base_values: self.base_values.clone(),
            modifiers: self.modifiers.clone(),
            finalizers: HashMap::new(), // Finalizers are currently lost on clone, but Player usually doesn't clone during gameplay
        }
    }
}

impl StatCalculator {
    pub fn new() -> Self {
        let mut finalizers: HashMap<Stat, Box<dyn StatFinalizer>> = HashMap::new();
        finalizers.insert(Stat::PAtk, Box::new(PAtkFinalizer));
        
        Self {
            base_values: HashMap::new(),
            modifiers: HashMap::new(),
            finalizers,
        }
    }

    pub fn calculate_all(&self) -> HashMap<Stat, f64> {
        let mut results = HashMap::new();
        
        // First pass: calculate stats that don't depend on others (or simple ones)
        // In a real implementation, we might need a dependency graph or multiple passes
        for &stat in &[Stat::Str, Stat::Int, Stat::Dex, Stat::Wit, Stat::Con, Stat::Men] {
            results.insert(stat, self.calculate_stat(stat, &results));
        }

        // Second pass: calculate dependent stats
        // This is a simplified version
        for stat_val in [Stat::PAtk, Stat::MAtk, Stat::PDef, Stat::MDef, Stat::MaxHp] { // Add more as needed
             results.insert(stat_val, self.calculate_stat(stat_val, &results));
        }

        results
    }

    pub fn calculate_stat(&self, stat: Stat, current_stats: &HashMap<Stat, f64>) -> f64 {
        let base = self.base_values.get(&stat).cloned().unwrap_or(0.0);
        let mut add = 0.0;
        let mut mul = 1.0;

        if let Some(mods) = self.modifiers.get(&stat) {
            for m in mods {
                match m {
                    Modifier::Add(v) => add += v,
                    Modifier::Mul(v) => mul *= v,
                }
            }
        }

        let finalizer = self.finalizers.get(&stat).map(|f| f.as_ref()).unwrap_or(&DefaultFinalizer);
        finalizer.finalize(base, add, mul, current_stats)
    }
}
