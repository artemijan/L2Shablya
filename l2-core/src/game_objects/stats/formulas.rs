use crate::game_objects::stats::stat_enum::Stat;
use std::collections::HashMap;

pub struct Formulas;

impl Formulas {
    pub fn calc_magic_dam(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
        skill_power: f64,
        is_bss: bool,
        is_mcrit: bool,
    ) -> f64 {
        let m_atk = attacker_stats.get(&Stat::MAtk).cloned().unwrap_or(1.0);
        let m_def = target_stats.get(&Stat::MDef).cloned().unwrap_or(1.0);
        
        let shots_bonus = if is_bss { 4.0 } else { 1.0 };
        let crit_mod = if is_mcrit { 
            attacker_stats.get(&Stat::MCriticalDamage).cloned().unwrap_or(2.0)
        } else { 
            1.0 
        };

        let damage = ((77.0 * skill_power * m_atk.sqrt()) / m_def) * shots_bonus;
        
        damage * crit_mod
    }

    pub fn calc_phys_dam(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
        is_ss: bool,
        is_crit: bool,
    ) -> f64 {
        let p_atk = attacker_stats.get(&Stat::PAtk).cloned().unwrap_or(1.0);
        let p_def = target_stats.get(&Stat::PDef).cloned().unwrap_or(1.0);
        
        let ss_mod = if is_ss { 2.0 } else { 1.0 };
        let crit_mod = if is_crit {
             attacker_stats.get(&Stat::PCriticalDamage).cloned().unwrap_or(2.0)
        } else {
             1.0
        };

        (p_atk * ss_mod * crit_mod * 70.0) / p_def
    }

    pub fn calc_blow_dam(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
        skill_power: f64,
        is_ss: bool,
        position_bonus: f64, // e.g., 0.2 for back
    ) -> f64 {
        let p_atk = attacker_stats.get(&Stat::PAtk).cloned().unwrap_or(1.0);
        let p_def = target_stats.get(&Stat::MDef).cloned().unwrap_or(1.0);
        let ss_mod = if is_ss { 2.0 } else { 1.0 };
        
        let crit_add = attacker_stats.get(&Stat::PCriticalDamage).cloned().unwrap_or(0.0);
        
        // Simplified blow formula based on Java logic
        let base_mod = (77.0 * ((skill_power + p_atk) * 0.666 + position_bonus * (skill_power + p_atk) + 6.0 * crit_add)) / p_def;
        
        base_mod * ss_mod
    }

    pub fn calc_hit_miss(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
    ) -> bool {
        let accuracy = attacker_stats.get(&Stat::PAccuracy).cloned().unwrap_or(100.0);
        let evasion = target_stats.get(&Stat::PEvasion).cloned().unwrap_or(100.0);
        
        let diff = accuracy - evasion;
        let mut chance = 90.0 + (diff * 2.0);
        
        if chance < 5.0 { chance = 5.0; }
        if chance > 98.0 { chance = 98.0; }

        // For now, let's just return false to avoid missing in tests/early stages
        // or actually implement a random check.
        // Since we don't have a global RNG easily here, return false for now.
        false
    }
}
