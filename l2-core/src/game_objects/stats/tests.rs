#[cfg(test)]
mod tests {
    use crate::game_objects::stats::stat_enum::Stat;
    use crate::game_objects::stats::calculator::{StatCalculator, Modifier};
    use crate::game_objects::stats::formulas::Formulas;
    use std::collections::HashMap;

    #[test]
    fn test_stat_calculation() {
        let mut calc = StatCalculator::new();
        calc.base_values.insert(Stat::Str, 40.0);
        calc.base_values.insert(Stat::PAtk, 100.0);
        calc.modifiers.entry(Stat::PAtk).or_default().push(Modifier::Add(20.0)); // 120
        calc.modifiers.entry(Stat::PAtk).or_default().push(Modifier::Mul(1.1)); // 132
        
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

        let mut target_stats = HashMap::new();
        target_stats.insert(Stat::MDef, 50.0);

        let dmg = Formulas::calc_magic_dam(&attacker_stats, &target_stats, 10.0, true, false);
        // ((77 * 10 * sqrt(100)) / 50) * 4 = ((77 * 10 * 10) / 50) * 4 = (7700 / 50) * 4 = 154 * 4 = 616
        assert_eq!(dmg, 616.0);
    }
}
