//! Skill effect classification and area target gathering.
//!
//! The mechanics implemented here follow the L2J reference behaviour documented in
//! `docs/skills/` (target selection in `02-target-selection.md`, damage in
//! `03-damage-formulas.md`, heals/buffs in `04-heal-buffs-debuffs.md`).

use crate::controller::GameController;
use crate::movement::calculate_distance;
use crate::pl_client::{FullStats, GetStats, PlayerClient};
use kameo::actor::ActorRef;
use l2_core::data::skills::{AffectScope, Skill};
use l2_core::game_objects::stats::calculator::Modifier;
use l2_core::game_objects::stats::stat_enum::Stat;
use std::sync::Arc;
use tracing::debug;

/// What a skill actually does when it lands, resolved for a concrete skill level.
#[derive(Debug, Clone)]
pub enum SkillAction {
    MagicDamage {
        power: f64,
    },
    PhysDamage {
        power: f64,
        crit_chance: f64,
    },
    Heal {
        power: f64,
    },
    HealPercent {
        percent: f64,
    },
    ManaHeal {
        power: f64,
    },
    ManaHealPercent {
        percent: f64,
    },
    /// Continuous stat modifiers (buff or debuff), see `Skill::abnormal_time_at`.
    Buff {
        mods: Vec<(Stat, Modifier)>,
    },
}

/// Maps a stat-effect handler name from the skill data to our [`Stat`] enum.
fn stat_by_effect_name(name: &str) -> Option<Stat> {
    Some(match name {
        "PAtk" => Stat::PAtk,
        "MAtk" => Stat::MAtk,
        "PhysicalDefence" => Stat::PDef,
        "MagicalDefence" => Stat::MDef,
        "PhysicalAttackSpeed" => Stat::PAtkSpd,
        "MagicalAttackSpeed" => Stat::MAtkSpd,
        "Speed" => Stat::MoveSpeed,
        "MaxHp" => Stat::MaxHp,
        "MaxMp" => Stat::MaxMp,
        "MaxCp" => Stat::MaxCp,
        "CriticalRate" => Stat::PCriticalRate,
        "CriticalDamage" => Stat::PCriticalDamage,
        "MagicCriticalRate" => Stat::MCriticalRate,
        "MagicCriticalDamage" => Stat::MCriticalDamage,
        "Accuracy" => Stat::PAccuracy,
        "PhysicalEvasion" => Stat::PEvasion,
        "MagicalEvasion" => Stat::MEvasion,
        "HpRegen" => Stat::RegenHp,
        "MpRegen" => Stat::RegenMp,
        "ShieldDefence" => Stat::ShieldDef,
        "ShieldDefenceRate" => Stat::ShieldRate,
        _ => return None,
    })
}

/// Resolves the skill's effect list into concrete actions for the given level.
/// Stat effects are folded into a single [`SkillAction::Buff`].
pub fn classify_effects(skill: &Skill, level: u8) -> Vec<SkillAction> {
    let mut actions = Vec::new();
    let mut buff_mods: Vec<(Stat, Modifier)> = Vec::new();
    for effect in skill.effects() {
        match effect.name.as_str() {
            "MagicalAttack"
            | "MagicalAttackMp"
            | "MagicalAttackByAbnormal"
            | "MagicalAttackRange" => actions.push(SkillAction::MagicDamage {
                power: effect.power(level),
            }),
            "PhysicalAttack"
            | "PhysicalAttackHpLink"
            | "PhysicalAttackSaveHp"
            | "PhysicalAttackWeaponBonus"
            | "PhysicalSoulAttack"
            | "Backstab"
            | "FatalBlow"
            | "EnergyAttack" => actions.push(SkillAction::PhysDamage {
                power: effect.power(level),
                crit_chance: effect
                    .critical_chance
                    .as_ref()
                    .and_then(|c| c.get(level))
                    .copied()
                    .unwrap_or(10.0),
            }),
            "Heal" => actions.push(SkillAction::Heal {
                power: effect.power(level),
            }),
            "HealPercent" => actions.push(SkillAction::HealPercent {
                percent: effect.power(level),
            }),
            "ManaHeal" | "ManaHealByLevel" => actions.push(SkillAction::ManaHeal {
                power: effect.power(level),
            }),
            "ManaHealPercent" => actions.push(SkillAction::ManaHealPercent {
                percent: effect.power(level),
            }),
            name => {
                if let Some(stat) = stat_by_effect_name(name) {
                    let amount = effect.amount(level);
                    let modifier = if effect.is_percent(level) {
                        Modifier::Mul(1.0 + amount / 100.0)
                    } else {
                        Modifier::Add(amount)
                    };
                    buff_mods.push((stat, modifier));
                } else {
                    debug!("Unhandled skill effect '{name}' of skill {}", skill.id);
                }
            }
        }
    }
    if !buff_mods.is_empty() {
        actions.push(SkillAction::Buff { mods: buff_mods });
    }
    actions
}

pub struct AffectedTarget {
    pub id: i32,
    pub actor: ActorRef<PlayerClient>,
    pub stats: FullStats,
}

/// Expands the main target into the final list of affected creatures, according to
/// the skill's affect scope (evaluated at skill launch, like in retail):
/// - `SINGLE`/`NONE` — the main target only;
/// - `POINT_BLANK` — everyone within `affectRange` around the **caster**;
/// - `RANGE` — the main target plus everyone within `affectRange` around it;
/// - other scopes currently fall back to single target.
///
/// The caster is never affected by its own area skill unless it is the main target.
#[allow(clippy::too_many_arguments)]
pub async fn gather_affected_targets(
    controller: &Arc<GameController>,
    scope: AffectScope,
    affect_range: i32,
    affect_limit: i32,
    caster_id: i32,
    caster_pos: (i32, i32, i32),
    main_target: AffectedTarget,
) -> Vec<AffectedTarget> {
    let origin = match scope {
        AffectScope::POINT_BLANK => caster_pos,
        _ => (
            main_target.stats.x,
            main_target.stats.y,
            main_target.stats.z,
        ),
    };
    if !scope.is_area() {
        return vec![main_target];
    }

    let mut result = Vec::new();
    let main_target_id = main_target.id;
    match scope {
        AffectScope::RANGE | AffectScope::RANGE_SORT_BY_HP | AffectScope::POINT_BLANK => {
            if scope != AffectScope::POINT_BLANK {
                result.push(main_target);
            }
            for (id, actor) in controller.all_players() {
                if result.len() >= affect_limit.max(1) as usize {
                    break;
                }
                if id == main_target_id && scope != AffectScope::POINT_BLANK {
                    continue;
                }
                // Area skills never affect the caster unless it is the main target.
                if id == caster_id {
                    continue;
                }
                let Ok(stats) = actor.ask(GetStats).await else {
                    continue;
                };
                let dist =
                    calculate_distance(origin.0, origin.1, origin.2, stats.x, stats.y, stats.z)
                        .unwrap_or(f64::MAX);
                if dist <= f64::from(affect_range) {
                    result.push(AffectedTarget { id, actor, stats });
                }
            }
        }
        // TODO: PARTY/PLEDGE scopes need party/clan membership checks; fall back to
        // the main target for now.
        _ => result.push(main_target),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill_from_yaml(yaml: &str) -> Skill {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn classify_magic_attack() {
        let skill = skill_from_yaml(
            r#"
'@id': '1177'
'@toLevel': '5'
'@name': Wind Strike
effects:
  effect:
  - '@name': MagicalAttack
    power:
      value:
      - '@level': '1'
        $text: '12'
"#,
        );
        let actions = classify_effects(&skill, 1);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            SkillAction::MagicDamage { power } if (power - 12.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn classify_heal() {
        let skill = skill_from_yaml(
            r#"
'@id': '1011'
'@toLevel': '1'
'@name': Heal
effects:
  effect:
  - '@name': Heal
    power:
      $text: '48'
"#,
        );
        let actions = classify_effects(&skill, 1);
        assert!(matches!(
            actions[0],
            SkillAction::Heal { power } if (power - 48.0).abs() < f64::EPSILON
        ));
    }

    #[test]
    fn classify_buff_with_percent_and_flat_mods() {
        let skill = skill_from_yaml(
            r#"
'@id': '1068'
'@toLevel': '3'
'@name': Might
operateType:
  $text: A2
effects:
  effect:
  - '@name': PAtk
    amount:
      $text: '8'
    mode:
      $text: PER
  - '@name': Speed
    amount:
      $text: '20'
    mode:
      $text: DIFF
"#,
        );
        let actions = classify_effects(&skill, 1);
        assert_eq!(actions.len(), 1);
        let SkillAction::Buff { mods } = &actions[0] else {
            panic!("expected buff action");
        };
        assert_eq!(mods.len(), 2);
        assert!(
            matches!(mods[0], (Stat::PAtk, Modifier::Mul(m)) if (m - 1.08).abs() < f64::EPSILON)
        );
        assert!(
            matches!(mods[1], (Stat::MoveSpeed, Modifier::Add(a)) if (a - 20.0).abs() < f64::EPSILON)
        );
    }

    #[test]
    fn unknown_effects_are_skipped() {
        let skill = skill_from_yaml(
            r#"
'@id': '999'
'@toLevel': '1'
'@name': Unknown
effects:
  effect:
  - '@name': SomethingExotic
"#,
        );
        assert!(classify_effects(&skill, 1).is_empty());
    }
}
