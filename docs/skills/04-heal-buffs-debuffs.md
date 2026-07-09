# Heals, Buffs and Debuffs

## Heal (`effecthandlers/Heal.java`, instant)

The `Heal` effect has one parameter: `power`. Final amount:

```
amount = power
if not static skill (isMagic != 2):
    amount += staticShotBonus + sqrt(mAtkMul * mAtk)
    amount *= target HEAL_EFFECT stat (heal receptiveness, default 1)
    amount += target HEAL_EFFECT_ADD stat
    x3 on heal critical (magic crit roll with skill magicCriticalRate)
amount = min(amount, maxRecoverableHp - currentHp)      // no overheal
```

Where for a mage with spiritshots:

```
staticShotBonus = skill.mpConsume      (x2.4 for blessed)
mAtkMul = bss ? 4 * SHOTS_BONUS : 2 * SHOTS_BONUS
```

without shots: `mAtkMul` = 1 (x4 for top S84 weapons, x2 S80), and `mAtkMul = mAtkMul + 1`.
So a plain heal is roughly `power + sqrt(2 * mAtk)`.

Messages: *"$s2 HP has been restored by $c1."* (self-heal: *"$s1 HP has been restored."*),
followed by a `StatusUpdate` broadcast of CUR_HP.

Potions/items add `ADDITIONAL_POTION_HP` and skip the m.atk scaling (static).

`HealOverTime` (regeneration-type) ticks `power` HP every `ticks * 666ms`.

## Mana heal (`ManaHeal`, `ManaHealByLevel`)

```
amount = power
if not static: amount = target MANA_CHARGE stat applied (recharger bonus)
amount = min(amount, maxRecoverableMp - currentMp)
```

*"$s2 MP has been restored by $c1."* / *"$s1 MP has been restored."*. The `ManaHealByLevel`
variant scales power by the *target's* level penalty.

## Buffs and debuffs (continuous effects)

A skill whose `operateType` is `A2`/`A3`/`DA1`... has **continuous** effects: stat modifiers
that live in the target's effect list for `abnormalTime` seconds.

### Effect application pipeline (`Skill.applyEffects`)

1. `calcEffectSuccess` roll (only for debuffs / skills with `activateRate`, see below); good
   buffs always land.
2. A `BuffInfo(caster, target, skill)` is created with `abnormalTime`.
3. Instant effects in the same skill (e.g. the damage of Stun Attack) are applied immediately.
4. Continuous effects are added to the target's **EffectList**:
   - Buffs with the same `abnormalType` replace each other if the new `abnormalLevel` is
     >= the old one (lower level → the new buff fails). E.g. Might lvl 1 can't override
     Might lvl 3, Haste from potion shares `SPEED_UP` with Wind Walk.
   - Buff count is limited (Interlude: 20 + divine inspiration); dances/songs have their own
     limit. Oldest buff is dropped when full.
   - Each effect's `pump()` adds its stat modifiers to the creature's stat calculator, then
     stats are recalculated and broadcast (`UserInfo`/`CharInfo`, `AbnormalStatusUpdate` for
     the buff bar, `ShortBuffStatusUpdate` for the first slot).
5. When `abnormalTime` expires (`BuffFinishTask`), effects `onExit`, modifiers are removed,
   stats recalculated, *"$s1 has worn off."* message sent.

### Stat effects (`AbstractStatEffect` subclasses)

Most buffs are declared with an effect name = stat and two params:

```xml
<effect name="PAtk">     <!-- Might -->
    <amount><value level="1">8</value>...</amount>
    <mode>PER</mode>     <!-- PER = percent (multiply by 1+amount/100), DIFF = flat add -->
</effect>
```

Common effect names: `PAtk`, `MAtk`, `PhysicalDefence`, `MagicalDefence`,
`PhysicalAttackSpeed`, `MagicalAttackSpeed`, `Speed`, `MaxHp`, `MaxMp`, `MaxCp`,
`CriticalRate`, `CriticalDamage`, `Accuracy`, `PhysicalEvasion`, `HpRegen`, `MpRegen`,
`MagicMpCost`, `VampiricAttack`, ... Debuff variants use the same handlers with negative
amounts.

`mode`:
- `DIFF` → `stat.add += amount` (e.g. Wind Walk: Speed +20)
- `PER` → `stat.mul *= 1 + amount/100` (e.g. Might: PAtk +8%)

### Abnormal (dis)ability effects

`BlockActions` (stun), `BlockMove` (root), `Sleep`, `Silence` (`BlockSkill` by magic type),
`Paralyze`, `Fear`, ... — these set effect flags checked by the engine instead of stats.

### Debuff landing chance (`Formulas.calcEffectSuccess`)

Rolled once per target for the whole skill (uses skill's `activateRate`; `-1` → always lands):

```
magicLevel   = skill.magicLevel (or targetLevel + 3 if unset)
baseMod      = (magicLevel - targetLevel + 3) * lvlBonusRate + activateRate + 30 - targetBasicPropertyResist
rate         = baseMod * elementMod * traitMod * buffDebuffResistMod
finalRate    = clamp(rate, minChance(default 10), maxChance(default 90)) * basicPropertyResistBonus
lands if finalRate > rnd(100)
```

- `targetBasicPropertyResist` — target's `ABNORMAL_RESIST_PHYSICAL` or `_MAGICAL` stat
  depending on the skill's `basicProperty`.
- Mesmerizing debuffs (stun/sleep/etc.) build up **resist levels** on repeated application
  (`BasicPropertyResist`): 2nd within 15s ⇒ 60% rate, 3rd ⇒ 30%, then immune; resets 15s
  after the last application.
- Failure sends *"$c1 has resisted your $s2."* and `ExMagicAttackInfo.RESISTED`.
- Debuffs can also be blocked by `AbnormalShieldBlocks` (Celestial-type shields) and
  reflected (`calcBuffDebuffReflection`, `REFLECT_SKILL_MAGIC/PHYSIC` stats) — a reflected
  debuff applies its continuous effects to the caster instead.

### Abnormal time

```
time = skill.abnormalTime (seconds); -1 for passives/toggles (infinite)
skill mastery on the caster ⇒ time *= 2
```

The remaining time is shown in the buff bar via `AbnormalStatusUpdate` (party members get
`PartySpelled`, the first short buff `ShortBuffStatusUpdate`).

### Toggles

Toggle skills (`A3`/`T`) don't roll success and don't expire; they consume MP per tick
(`mpPerChanneling`/`itemConsume`) and are stopped manually or when out of resources.

### Buff slots & dispel

- `abnormalType` is the dedup key; `BlockAbnormalSlot` effects give immunity against whole
  slots.
- Dispel/steal effects (`DispelBySlot`, `DispelBySlotProbability`, cancel):
  `chance = rate + (cancelMagicLvl - buffMagicLvl) * 2 + buffTime/120 * RESIST_DISPEL_BUFF`,
  clamped 25..75%, applied per buff from the newest to the oldest.

## PvP flagging rules (`SkillCaster.callSkill`)

- Casting a **bad** skill on a playable → attacker gets flagged (`updatePvPStatus(target)`).
- Casting a **good** skill on a flagged/karma player (or a monster) → caster gets flagged
  (unless self-cast).
- Bad skills on NPCs add `-effectPoint` hate and put the caster on the NPC's attack list.
