# Skill Model and Casting Flow

## Skill data model

Skills are defined in `data/stats/skills/*.xml` (converted to YAML for this project). Every
skill has an id, a name and 1..N levels. Any parameter can be either a single value or a
per-level table:

```xml
<mpConsume>
    <value level="1">7</value>
    <value level="2">7</value>
    <value level="3">8</value>
</mpConsume>
```

The most important parameters:

| Parameter | Meaning |
|-----------|---------|
| `operateType` | `A1` active instant-effect skill (attacks, heals), `A2` active with continuous effect (buffs/debuffs, duration = `abnormalTime`), `A3` active toggle, `P` passive, `CA1`/`CA5` channeling, `DA1`/`DA2` dance/song, `T` toggle |
| `isMagic` | `0` physical skill, `1` magic (uses M.Atk/M.Def, cast speed scales with casting spd), `2` static (potions etc., no stat scaling), `3` music, `4` item skill |
| `castRange` | max distance (units) between caster and the *main* target to start casting; `-1` = no range (self, auras) |
| `effectRange` | max distance at skill *launch* — if the target moved beyond it mid-cast, the cast is cancelled |
| `hitTime` | base cast animation time in ms; scaled by caster's attack/cast speed (see below) |
| `coolTime` | small "recovery" animation time after the skill lands, scaled like hitTime |
| `reuseDelay` | cooldown in ms; `staticReuse` makes it independent of cast speed stats |
| `reuseDelayGroup` | skills sharing this group share their cooldown |
| `mpConsume` / `mpInitialConsume` | MP consumed at launch / at cast start |
| `hpConsume` | HP consumed at launch |
| `effectPoint` | "aggression" points; **sign defines whether a skill is good or bad**: `effectPoint < 0` → offensive (`isBad()`), `> 0` → friendly. Also used as aggro amount for NPC hate lists |
| `magicLevel` | level used in level-difference modifiers (effect land rate, magic failure) |
| `magicCriticalRate` | base magic critical chance (usually 5) |
| `abnormalType` / `abnormalLevel` / `abnormalTime` | buff slot type, its strength and duration in **seconds** (see doc 04) |
| `targetType` / `affectScope` / `affectObject` / `affectRange` / `affectLimit` | targeting (see doc 02) |
| `attributeType` / `attributeValue` | elemental attack attribute of the skill (FIRE/WATER/WIND/EARTH/HOLY/DARK) |
| `basicProperty` | which resist stat defends against the effect (`PHYSICAL`, `MAGIC`, `NONE`) |
| `effects` | list of effect handlers with parameters, e.g. `MagicalAttack{power}`, `Heal{power}`, `PAtk{amount, mode}` |

A skill is **bad** (offensive) iff `effectPoint < 0`. This is used everywhere: PvP flagging,
peace-zone checks, "you cannot buff monsters" checks, auto-attack stance, etc.

## Request flow (`RequestMagicSkillUse`)

Client packet: `skillId: i32`, `ctrlPressed: i32` (force attack), `shiftPressed: u8` (don't move).

1. The player must **know** the skill (`player.getKnownSkill`), otherwise `ActionFailed`.
2. If the skill is disabled (on cooldown) → system message *"$s1 is not available at this time:
   being prepared for reuse."* (id 48).
3. `useMagic` checks: not dead, not casting (or enqueue as `queuedSkill`), skill conditions,
   not muted (physical/magical mute), enough MP/HP, item requirements.
4. Target is resolved via the skill's `targetType` handler (see doc 02). If it returns null →
   `ActionFailed`.
5. You cannot cast a **good** skill (`effectPoint > 0`) on a monster without holding Ctrl
   → *Invalid target*.
6. If `castRange > 0` and the target is farther → the AI moves the caster into range and
   retries (unless Shift is pressed → *"The distance is too far..."*).
7. Geodata line-of-sight must exist between caster and target → otherwise *Cannot see target*.

## Casting state machine (`SkillCaster`)

Casting is a 3-phase scheduled task:

```
phase 0: startCasting()   --(wait hitTime)-->
phase 1: launchSkill()    --(wait cancelTime)-->
phase 2: finishSkill()    --(wait coolTime)--> done
```

Skills that are `isAbnormalInstant`, `isWithoutAction`, toggles, or SIMULTANEOUS casts skip
the phases and apply immediately.

### Phase 0 — startCasting

- Register the reuse (cooldown) and disable the skill: `reuseDelay` is taken from the stat
  system (`getReuseTime`), can be modified by skill mastery (reuse = 100ms).
- Stop movement; face the target (send `MoveToPawn`/`ExRotation`).
- Consume `mpInitialConsume` (send `StatusUpdate CUR_MP`); abort if not enough (*Not enough MP*).
- Broadcast **`MagicSkillUse`** packet with `displayedCastTime = hitTime + cancelTime` and the
  reuse info — this shows the casting bar & animation to everyone.
- Send *"You use $s1."* system message and `SetupGauge` (blue bar) to the caster.
- Consume reagent items, fame, clan reputation if configured.

### Phase 1 — launchSkill (after `hitTime`)

- If `effectRange > 0` and the target moved out of it → cancel (*The distance is too far and
  so the casting has been cancelled*).
- Gather the final list of **affected targets** via the `affectScope` handler (doc 02). Area
  skills collect everyone in the area *at launch time*, not at cast start.
- Broadcast **`MagicSkillLaunched`** with the list of affected target ids.

### Phase 2 — finishSkill (after `cancelTime`)

- Consume `mpConsume` and `hpConsume` (send `StatusUpdate`), abort if not enough.
- `callSkill`: apply the skill's effects on every gathered target (doc 03/04), trigger weapon
  on-magic-skill abilities, raid curses, PvP flag updates, aggro (`effectPoint`), notify AI of
  the attack.
- If the skill is bad and has a target ≠ caster → start auto-attack stance.

If any phase fails, casting stops: broadcast `MagicSkillCanceled` and send `ActionFailed`.

## Timing formulas

Base times come from the skill (`hitTime`, `coolTime`, in ms). They are scaled by the caster's
speed (`Formulas.calcAtkSpd`):

```
magic:    time = hitTime / mAtkSpd * 333
physical: time = hitTime / pAtkSpd * 300
```

333 is the base M.Atk. speed and 300 the base P.Atk. speed. E.g. Wind Strike (hitTime 4000)
with 333 cast speed takes 4.0s; with 666 it takes 2.0s.

Notes:

- Spiritshots additionally speed up magic casts (factor ≈ 1.4 in the skill-time factor
  formula: `factor = mAtkSpdMul + mAtkSpdMul * 0.4`).
- Static skills (`isMagic=2`, potions) and channeling skills are **not** scaled.
- The cancel time (time between launch and finish) is
  `max(hitCancelTime * 1000 / factor, 500)`; the 500ms constant is `SKILL_LAUNCH_TIME` —
  effects land approximately 500ms after the launch packet.
- Client-side casting bar shows `hitTime + cancelTime`.

## Reuse (cooldown)

- Reuse starts at **cast start** (phase 0), not at finish.
- `reuseDelay` can be scaled by stats (`MAGIC_REUSE_RATE`, `P_REUSE`, ...) unless
  `staticReuse` is set.
- If `reuseDelayGroup > 0`, all skills with the same group are disabled together (e.g.
  healing potions). The client is informed via the reuse fields of `MagicSkillUse` and by
  `SkillCoolTime` packets.
- Skill mastery (a passive that procs on cast) sets reuse to 100ms and shows
  *"A skill is ready to be used again."*

## MP consumption summary

| When | Amount | On failure |
|------|--------|-----------|
| Phase 0 (cast start) | `mpInitialConsume` | *Not enough MP*, cast aborted |
| Phase 2 (finish) | `mpConsume` (scaled by `MAGICAL_MP_CONSUME_RATE` etc., dances use `DANCE_MP_CONSUME_RATE`) | *Not enough MP*, effects not applied |
