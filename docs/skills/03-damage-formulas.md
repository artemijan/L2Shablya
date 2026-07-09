# Damage Formulas

All formulas are from `gameserver/model/stats/Formulas.java` and the effect handlers
(`MagicalAttack.java`, `PhysicalAttack.java`). Damage effects are *instant* effects executed
in `finishSkill` for every affected target.

## Magic skill damage (`Formulas.calcMagicDam`)

Used by the `MagicalAttack` effect (`power` parameter comes from the skill effect data).

```
shotsBonus = bss ? 4 * SHOTS_BONUS : sps ? 2 * SHOTS_BONUS : 1     // spiritshot multiplier on mAtk
damage = 77 * (power + SKILL_POWER_ADD) * sqrt(mAtk) / mDef * shotsBonus

damage *= critMod            // magic critical: 2 * MAGIC_CRITICAL_DAMAGE * DEF_MCRIT_DAMAGE (≈ x2..x4)
damage *= generalTraitMod    // trait attack/defence (== 0 treated as 1)
damage *= weaknessMod
damage *= attributeMod       // elemental, see below
damage *= randomMod          // 1 ± randomDamage/100 (weapon random damage, default 5 ⇒ ±5%)
damage *= pvpPveMod
damage *= MAGICAL_SKILL_POWER stat (default 1)
```

- `mAtk` — caster's M.Atk (buffed, including active shots multipliers), `mDef` — target M.Def.
- `sps`/`bss` — spiritshots / blessed spiritshots charged and the skill uses them.

### Magic failure (`calcMagicSuccess`, config `ALT_GAME_MAGICFAILURES`)

Depends on the level difference (vs `magicLevel`) and magic accuracy vs magic evasion:

```
PvE: lvlModifier = 1.3 ^ (targetLevel - magicLevel or casterLevel)
PvP: mAccModifier from (mAcc - mEvasion): >-20 → 2, >-25 → 30, >-30 → 60, >-35 → 90, else 100
rate = 100 - round(mAccModifier * lvlModifier * targetModifier * MAGIC_SUCCESS_RES)
```

On failure: half damage (*"Your attack has failed."*) or full resist → damage = 1
(*"$c1 has resisted your $s2."*).

### Magic critical (`calcCrit` for magic skills)

```
rate = caster stat MAGIC_CRITICAL_RATE (base = skill magicCriticalRate, usually 5)
good skill or no target:            crit if min(rate, 320) > rnd(1000)
bad skill:  finalRate = target DEFENCE_MAGIC_CRITICAL_RATE(rate) + DEF..RATE_ADD
            both lvl >= 78: finalRate += sqrt(casterLvl) + (casterLvl - targetLvl)/25, cap 320
            else cap 200
crit if finalRate > rnd(1000)      // i.e. rate is in tenths of a percent
```

Magic crit damage multiplier: `2 * MAGIC_CRITICAL_DAMAGE * DEFENCE_MAGIC_CRITICAL_DAMAGE`
(Interlude-era default ≈ x3 was used for mana damage; through `calcCritDamage` the base is x2).

## Physical skill damage (`PhysicalAttack` effect)

```
attack  = pAtk * pAtkMod * randomMod (+ position bonus)
defence = pDef * pDefMod (+ shield, see below)

power      = skill power (+ SKILL_POWER_ADD)
weaponMod  = ranged weapon ? 70 : 77
rangedBonus= ranged ? attack + power : 0
baseMod    = weaponMod * (attack * levelMod + power + rangedBonus) / defence

damage = baseMod * ssmod * critMod * weaponTraitMod * generalTraitMod
         * weaknessMod * attributeMod * pvpPveMod * randomMod
         * PHYSICAL_SKILL_POWER stat
```

- `ssmod` = `2 * SHOTS_BONUS` with soulshots, else 1.
- `critMod` = `calcCritDamage` (2 × crit damage stats) when the skill crits; physical skill
  crit chance is `criticalChance` effect param scaled by STR bonus and `CRITICAL_RATE_SKILL`,
  constrained to 5..90%.
- `levelMod` — caster level modifier (`(level + 89) / 100` for players).
- Skill evasion (`calcSkillEvasion`) can dodge the whole skill (*"$c1 dodged the attack"*).

## Blow damage (daggers, `calcBlowDamage`)

```
77 * ((power + pAtk) * 0.666 * cdBonus * cdPosHalf * cdVulnHalf * ss
      + positionMod(back 0.2 / side 0.05 / front 0) * (power + pAtk) * random
      + 6 * cdAddPatk) / pDef
```

Blow success chance (`calcBlowSuccess`): `weaponCritRate * heightBonus * positionBonus *
(100+chanceBoost)/100 * BLOW_RATE`, capped by config (position bonus: back ×1.3, side ×1.1).

## Auto-attack damage (`calcAutoAttackDamage`)

```
attack = pAtk * random ± proximity bonus (behind +20% pAtk, side +5%)
critical part:      ((attack * cAtk * ssBonus + cAtkAdd) * critMod) * weaponMod
non-critical part:  attack * (1 - critMod) * ssBonus * weaponMod
weaponMod = ranged ? 154 : 77
damage = (critical + non-critical) / pDef * traits * attribute * pvpPve
```

`cAtk = 2 * CRITICAL_DAMAGE * DEF_CRITICAL_DAMAGE * position mods`, `ssBonus = 2 (2.15
blessed) * SHOTS_BONUS`, ranged crits use `critMod 0.5`.

### Auto-attack critical chance

`rate = DEX-based base crit (weapon crit rate) / 10 * positionBonus (front x1.0 / side x1.1 /
back x1.3) * heightBonus`, constrained 3..97%. High level (78+) adds
`sqrt(casterLvl) * (casterLvl - targetLvl) * 0.125`.

### Hit miss (`calcHitMiss`)

```
chance = (80 + 2 * (accuracy - evasion)) * 10        // per mille
chance *= hit condition bonus (position/darkness/rain)
clamped to [200, 980] ⇒ always 2%..20% miss window
miss if chance < rnd(1000)
```

## Shield block (`calcShldUse`)

Target must have a shield equipped and the attack must come from the front 120° arc
(or 360° with `PHYSICAL_SHIELD_ANGLE_ALL`):

```
shldRate = SHIELD_DEFENCE_RATE stat * CON bonus         (* 1.3 vs bows)
block:          rnd(100) < shldRate
perfect block:  additionally rnd(100) > 100 - 2*CON_bonus
```

Result: `SUCCEED` → target pDef += shield def; `PERFECT_BLOCK` → final damage = 1.

## Elemental attribute modifier (`calcAttributeBonus`)

```
diff = attackAttribute - defenceAttribute
diff > 0: min(1.025 + sqrt(diff³/2) * 0.0001, 1.25)
diff < 0: max(0.975 - sqrt(|diff|³/2) * 0.0001, 0.75)
diff == 0: 1.0
```

The skill's `attributeType/attributeValue` is added on top of the caster's weapon attribute.

## Trait modifiers

- `calcWeaponTraitBonus`: `max(0.22, 1 - targetDefence(weaponTrait))` — e.g. "sword
  vulnerability/resistance".
- `calcGeneralTraitBonus`: for skill traits (SHOCK, BLEED, HOLD...):
  `max(attackTrait - defenceTrait, 0.05)`; invulnerable trait → 0 (treated as 1 for raw
  damage, but blocks effect landing).

## PvP / PvE bonus (`calculatePvpPveBonus`)

```
playable vs playable: max(0.05, dragonDef * (1 + PVP_DMG_STAT - PVP_DEF_STAT)) * config class multipliers
vs NPC:               max(0.05, (1 + PVE_DMG*raid - PVE_DEF*raid) * highLevelPenalty)
```

High level NPC penalty: when target NPC is ≥ 2 levels above the player, damage/land rates are
scaled down by `NPC_DMG/SKILL_PENALTY` config tables.

## Applying damage

`Creature.doAttack(damage, target, skill, ...)`:

- Interrupts target cast with `calcAtkBreak` chance (`init 15 + sqrt(13*dmg) - MEN bonus`).
- `reduceCurrentHp` → HP down, `StatusUpdate` broadcast, death handling.
- System messages: attacker *"$c1 has inflicted $s3 damage on $c2."* (2261), victim
  *"$c1 has received $s3 damage from $c2."* (2262); crits show *"M-Critical"* / damage popups.
- Damage reflection (`REFLECT_DAMAGE_PERCENT`), vengeance (`calcCounterAttack`) and absorb
  (vampiric) are applied for melee/physical hits.
