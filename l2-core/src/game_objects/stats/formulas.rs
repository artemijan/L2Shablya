use crate::game_objects::stats::stat_enum::Stat;
use rand::RngExt;
use std::collections::HashMap;

pub struct Formulas;

fn stat(stats: &HashMap<Stat, f64>, s: Stat, default: f64) -> f64 {
    stats.get(&s).copied().unwrap_or(default)
}

impl Formulas {
    /// `1 ± randomDamage%` weapon spread (retail default 5 ⇒ ±5%).
    pub fn random_damage_multiplier(attacker_stats: &HashMap<Stat, f64>) -> f64 {
        let spread = stat(attacker_stats, Stat::RandomDamage, 5.0);
        1.0 + rand::rng().random_range(-spread..=spread) / 100.0
    }

    /// Magic critical roll. `base_rate` is the skill's `magicCriticalRate` (usually 5, in %),
    /// overridden by the caster's `MCriticalRate` stat when present. Rolled per mille,
    /// capped at 32% like in retail.
    pub fn calc_magic_crit(attacker_stats: &HashMap<Stat, f64>, base_rate: f64) -> bool {
        let rate_per_mille = stat(attacker_stats, Stat::MCriticalRate, base_rate) * 10.0;
        rate_per_mille.min(320.0) > f64::from(rand::rng().random_range(0..1000))
    }

    /// Physical skill critical roll: `criticalChance` effect param scaled by the caster's
    /// `PCriticalRate` multipliers, constrained to 5..90% like retail skill crits.
    pub fn calc_phys_skill_crit(base_chance: f64) -> bool {
        let rate = base_chance.clamp(5.0, 90.0);
        rate > f64::from(rand::rng().random_range(0..100))
    }

    /// Magic damage formula (`Formulas.calcMagicDam` in L2J):
    /// `77 * power * sqrt(mAtk) / mDef * shotsBonus * critMod * randomMod`.
    pub fn calc_magic_dam(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
        skill_power: f64,
        sps: bool,
        bss: bool,
        is_mcrit: bool,
    ) -> f64 {
        let m_atk = stat(attacker_stats, Stat::MAtk, 1.0).max(1.0);
        let m_def = stat(target_stats, Stat::MDef, 1.0).max(1.0);

        let shots_bonus = if bss {
            4.0
        } else if sps {
            2.0
        } else {
            1.0
        };
        // calcCritDamage: 2 * MAGIC_CRITICAL_DAMAGE * DEFENCE_MAGIC_CRITICAL_DAMAGE
        let crit_mod = if is_mcrit {
            stat(attacker_stats, Stat::MCriticalDamage, 2.0)
        } else {
            1.0
        };

        let damage = (77.0 * skill_power * (m_atk * shots_bonus).sqrt()) / m_def;
        (damage * crit_mod * Self::random_damage_multiplier(attacker_stats)).max(1.0)
    }

    /// Physical skill damage (`PhysicalAttack` effect handler):
    /// `77 * (pAtk * random + power) / pDef`, doubled shots/crit multipliers.
    pub fn calc_phys_skill_dam(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
        skill_power: f64,
        is_ss: bool,
        is_crit: bool,
    ) -> f64 {
        let p_atk = stat(attacker_stats, Stat::PAtk, 1.0).max(1.0);
        let p_def = stat(target_stats, Stat::PDef, 1.0).max(1.0);

        let ss_mod = if is_ss { 2.0 } else { 1.0 };
        let crit_mod = if is_crit {
            stat(attacker_stats, Stat::PCriticalDamage, 2.0)
        } else {
            1.0
        };
        let random_mod = Self::random_damage_multiplier(attacker_stats);

        (77.0 * (p_atk * random_mod + skill_power) / p_def * ss_mod * crit_mod).max(1.0)
    }

    /// Heal amount (`Heal` effect handler): `power + sqrt(mAtkMul * mAtk)`,
    /// tripled on heal critical. Returns `(amount, was_critical)`. The caller must cap
    /// the amount at the target's missing HP.
    pub fn calc_heal(
        attacker_stats: &HashMap<Stat, f64>,
        skill_power: f64,
        magic_crit_rate: f64,
        sps: bool,
        bss: bool,
    ) -> (f64, bool) {
        let m_atk = stat(attacker_stats, Stat::MAtk, 0.0).max(0.0);
        let m_atk_mul = if bss {
            4.0
        } else if sps {
            2.0
        } else {
            1.0
        };
        let mut amount = skill_power + (m_atk_mul * m_atk).sqrt();
        let crit = Self::calc_magic_crit(attacker_stats, magic_crit_rate);
        if crit {
            amount *= 3.0;
        }
        (amount, crit)
    }

    /// Auto-attack damage: `77 * pAtk * random / pDef` with soulshot and crit multipliers.
    pub fn calc_phys_dam(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
        is_ss: bool,
        is_crit: bool,
    ) -> f64 {
        let p_atk = stat(attacker_stats, Stat::PAtk, 1.0).max(1.0);
        let p_def = stat(target_stats, Stat::PDef, 1.0).max(1.0);

        let ss_mod = if is_ss { 2.0 } else { 1.0 };
        let crit_mod = if is_crit {
            stat(attacker_stats, Stat::PCriticalDamage, 2.0)
        } else {
            1.0
        };

        (p_atk * ss_mod * crit_mod * 77.0 / p_def * Self::random_damage_multiplier(attacker_stats))
            .max(1.0)
    }

    pub fn calc_blow_dam(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
        skill_power: f64,
        is_ss: bool,
        position_bonus: f64, // e.g., 0.2 for back
    ) -> f64 {
        let p_atk = stat(attacker_stats, Stat::PAtk, 1.0);
        let p_def = stat(target_stats, Stat::PDef, 1.0).max(1.0);
        let ss_mod = if is_ss { 2.0 } else { 1.0 };

        let crit_add = stat(attacker_stats, Stat::PCriticalDamage, 0.0);

        // Simplified blow formula based on Java logic
        let base_mod = (77.0
            * ((skill_power + p_atk) * 0.666
                + position_bonus * (skill_power + p_atk)
                + 6.0 * crit_add))
            / p_def;

        base_mod * ss_mod
    }

    /// Hit miss roll (`Formulas.calcHitMiss`):
    /// `chance = (80 + 2 * (accuracy - evasion)) * 10` per mille, clamped to 200..980.
    pub fn calc_hit_miss(
        attacker_stats: &HashMap<Stat, f64>,
        target_stats: &HashMap<Stat, f64>,
    ) -> bool {
        let accuracy = stat(attacker_stats, Stat::PAccuracy, 100.0);
        let evasion = stat(target_stats, Stat::PEvasion, 100.0);

        let chance = ((80.0 + 2.0 * (accuracy - evasion)) * 10.0).clamp(200.0, 980.0);
        chance < f64::from(rand::rng().random_range(0..1000))
    }

    /// Continuous (debuff) effect landing chance (`Formulas.calcEffectSuccess`), simplified:
    /// good skills always land; `activate_rate == -1` always lands.
    pub fn calc_effect_success(
        magic_level: i32,
        target_level: i32,
        activate_rate: i32,
        lvl_bonus_rate: i32,
    ) -> bool {
        if activate_rate == -1 {
            return true;
        }
        let base = f64::from((magic_level - target_level + 3) * lvl_bonus_rate.max(1))
            + f64::from(activate_rate)
            + 30.0;
        let rate = base.clamp(10.0, 90.0);
        rate > f64::from(rand::rng().random_range(0..100))
    }
}
