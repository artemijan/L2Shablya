# Skill Mechanics Documentation

This documentation describes the Lineage 2 (Interlude Classic) skill mechanics as implemented
by the reference Java server (L2J Mobius, `/Users/artem/dev/l2/interlude_classic`). It is the
specification for the Rust implementation in this repository.

| File | Contents |
|------|----------|
| [01-skill-model-and-casting.md](01-skill-model-and-casting.md) | Skill data model, operate types, the 3-phase casting flow, cast/reuse timing, MP/HP consumption |
| [02-target-selection.md](02-target-selection.md) | Target types, affect scopes (single vs. area), affect objects, ctrl/shift semantics |
| [03-damage-formulas.md](03-damage-formulas.md) | Magic/physical skill damage, blow damage, auto-attacks, criticals, shield, evasion, attribute/trait/PvP-PvE modifiers |
| [04-heal-buffs-debuffs.md](04-heal-buffs-debuffs.md) | Heal & mana heal formulas, buff/debuff application, abnormal types & times, effect success chance, stat modifiers |

## Key Java reference files

- `gameserver/network/clientpackets/RequestMagicSkillUse.java` — entry point of a skill request
- `gameserver/model/skill/SkillCaster.java` — the casting state machine
- `gameserver/model/skill/Skill.java` — skill data + target resolution + effect application
- `gameserver/model/stats/Formulas.java` — all combat formulas
- `dist/game/data/scripts/handlers/effecthandlers/*.java` — instant & continuous effects (Heal, MagicalAttack, PhysicalAttack, stat buffs, ...)
- `dist/game/data/scripts/handlers/targethandlers/*.java` — target type handlers
- `dist/game/data/scripts/handlers/targethandlers/affectscope/*.java` — area-of-effect scopes
- `dist/game/data/stats/skills/*.xml` — skill definitions (source of `config/data/stats/skills/*.yaml`)
