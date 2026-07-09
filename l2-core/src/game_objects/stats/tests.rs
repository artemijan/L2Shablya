#[cfg(test)]
mod tests {
    use crate::game_objects::stats::calculator::{Modifier, StatCalculator};
    use crate::game_objects::stats::formulas::Formulas;
    use crate::game_objects::stats::stat_enum::Stat;
    use std::collections::HashMap;

    #[test]
    fn test_stat_calculation() {
        let mut calc = StatCalculator::new();
        calc.base_values.insert(Stat::Str, 40.0);
        calc.base_values.insert(Stat::PAtk, 100.0);
        calc.modifiers
            .entry(Stat::PAtk)
            .or_default()
            .push(Modifier::Add(20.0)); // 120
        calc.modifiers
            .entry(Stat::PAtk)
            .or_default()
            .push(Modifier::Mul(1.1)); // 132

        let stats = calc.calculate_all();
        let patk = stats.get(&Stat::PAtk).unwrap();

        // Base(100) + Add(20) = 120
        // 120 * Mul(1.1) = 132
        // STR is 40, so STR bonus is 1.0 (in our simplified PAtkFinalizer)
        assert_eq!(*patk, 132.0);

        // Test with higher STR
        calc.base_values.insert(Stat::Str, 50.0);
        let stats2 = calc.calculate_all();
        let patk2 = stats2.get(&Stat::PAtk).unwrap();
        // STR 50 => bonus = 1.0 + (50-40)*0.05 = 1.5
        // 132 * 1.5 = 198.0
        assert_eq!(*patk2, 198.0);
    }

    #[test]
    fn test_damage_formulas() {
        let mut attacker_stats = HashMap::new();
        attacker_stats.insert(Stat::MAtk, 100.0);
        attacker_stats.insert(Stat::MCriticalDamage, 2.0);
        // no weapon spread so the damage roll is deterministic
        attacker_stats.insert(Stat::RandomDamage, 0.0);

        let mut target_stats = HashMap::new();
        target_stats.insert(Stat::MDef, 50.0);

        // no shots, no crit: 77 * 10 * sqrt(100) / 50 = 154
        let dmg =
            Formulas::calc_magic_dam(&attacker_stats, &target_stats, 10.0, false, false, false);
        assert!((dmg - 154.0).abs() < 1e-6);

        // blessed spiritshots multiply m.atk by 4 => sqrt doubles: 308
        let dmg_bss =
            Formulas::calc_magic_dam(&attacker_stats, &target_stats, 10.0, false, true, false);
        assert!((dmg_bss - 308.0).abs() < 1e-6);

        // magic crit multiplies by MCriticalDamage stat (2.0)
        let dmg_crit =
            Formulas::calc_magic_dam(&attacker_stats, &target_stats, 10.0, false, false, true);
        assert!((dmg_crit - 308.0).abs() < 1e-6);
    }

    #[test]
    fn test_heal_formula() {
        let mut attacker_stats = HashMap::new();
        attacker_stats.insert(Stat::MAtk, 100.0);
        attacker_stats.insert(Stat::MCriticalRate, 0.0); // never crit
        let (amount, crit) = Formulas::calc_heal(&attacker_stats, 50.0, 0.0, false, false);
        assert!(!crit);
        // 50 + sqrt(100) = 60
        assert!((amount - 60.0).abs() < 1e-6);
    }
}
