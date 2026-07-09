# Target Selection

Target selection happens in two steps:

1. **`targetType`** — validates/resolves the *main* target from what the player currently has
   selected (at cast start).
2. **`affectScope`** — expands the main target into the final list of affected creatures
   (at skill launch). `affectObject` further filters which creatures inside the scope are
   eligible.

```
main target = TargetTypeHandler(caster, selectedTarget, skill, ctrl, shift)
targets     = AffectScopeHandler(caster, mainTarget, skill) filtered by AffectObjectHandler
```

## Target types (`targetType`)

The important ones (full enum: `model/skill/targets/TargetType.java`):

| Type | Main target | Rules |
|------|-------------|-------|
| `SELF` | the caster | Bad self-skills are rejected in peace zone |
| `TARGET` | current selection | Must be a creature. You can always target yourself. Shift + out of `castRange` → cancel. Requires geodata visibility |
| `ENEMY` | current selection | Must be a creature, **not self**, not dead (unless `isStayAfterDeath`). Allowed if target `isAutoAttackable(caster)` **or Ctrl is pressed**. Rejected in peace zone. Siege allies can't be force-attacked |
| `ENEMY_ONLY` | current selection | Like `ENEMY` but Ctrl does not override: the target must be a real enemy (auto-attackable / at war / duel / flagged) |
| `ENEMY_NOT` | current selection | Friendly targets only (not auto-attackable) |
| `OTHERS` | current selection | Anyone except the caster |
| `GROUND` | the caster (position from `RequestExMagicSkillUseGround`) | Area is centered on the stored ground location |
| `MY_PARTY` | the caster | Scope restricted to caster's party |
| `SUMMON` / `OWNER_PET` | own summon / pet owner | |
| `PC_BODY` / `NPC_BODY` | a corpse | For resurrection / sweep / drain-type skills |
| `NONE` | the caster | Used together with aura scopes |
| `DOOR_TREASURE`, `HOLYTHING`, `ARTILLERY`, `ADVANCE_BASE`, `FORTRESS_FLAGPOLE`, `WYVERN_TARGET`, `ITEM`, `MY_MENTOR` | special objects | |

Common checks performed by the handlers:

- **Invalid target** (sys msg 109) when the selection is missing/of a wrong kind.
- **Dead check**: bad skills can't be cast on dead targets (except corpse-target skills).
- **Peace zone**: bad skills are rejected when either side is in a peace zone
  (*"You cannot use skills that may harm other players in here."*).
- **Ctrl (force use)**: allows attacking non-flagged players (`ENEMY`), and is *required* to
  cast good skills on monsters (`skill.effectPoint > 0 && target.isMonster() && !ctrl` →
  Invalid target).
- **Shift (dontMove)**: if the target is beyond `castRange`, do not walk to it — cancel with
  *"The distance is too far and so the casting has been cancelled."*
- **Geodata**: `canSeeTarget` must be true, else *Cannot see target.*

## Affect scopes (`affectScope`)

Evaluated at launch. `affectRange` is the radius, `affectLimit` ("min-max" in the XML) caps
the number of targets (0 = unlimited).

| Scope | Area | Origin |
|-------|------|--------|
| `SINGLE` | main target only | — |
| `POINT_BLANK` | sphere with radius `affectRange` | **caster** (self-centered AoE, e.g. aura attacks) |
| `RANGE` | sphere with radius `affectRange` | **main target** (classic AoE nukes) |
| `RING_RANGE` | donut: creatures between `fanRange[2]` (start) and `fanRange[3]` (end radius) | caster |
| `FAN` / `FAN_PB` | circular sector: heading ± `fanRange[1]` degrees, radius `fanRange[2]`, angle `fanRange[3]` | caster (FAN uses cast direction to target, FAN_PB caster heading) |
| `SQUARE` / `SQUARE_PB` | rectangle `affectRange` long/wide | target / caster |
| `PARTY` | party members (incl. summons) within `affectRange` | main target |
| `PARTY_PLEDGE` | party + clan members within range | main target |
| `PLEDGE` | clan members within range | main target |
| `DEAD_PARTY`, `DEAD_PLEDGE`, `DEAD_PARTY_PLEDGE`, `DEAD_UNION` | as above but **dead** members (mass resurrection) | |
| `SUMMON_EXCEPT_MASTER` | caster's summons | |

Shared filter rules (see `affectscope/*.java`):

- Dead creatures are excluded (except corpse target types / DEAD_* scopes).
- The caster is **not** affected by its own RANGE skill unless it is the main target
  ("Range skills appear to not affect you unless you are the main target").
- Every affected creature must be visible (geodata) from the scope origin.
- `affectLimit` stops collection once reached (Java reads it as `affectLimit[0] + rnd(affectLimit[1])`
  from the "min-max" pair).
- For `GROUND` target type, the origin is the stored ground location instead of the target.

## Affect objects (`affectObject`)

Filters *which kind* of creature inside the scope is affected:

| Value | Affected |
|-------|----------|
| `ALL` | everyone |
| `CLAN` | same clan (for NPCs: same clan id in template) |
| `FRIEND` | non-enemies: same party/clan/ally/command channel, non-flagged non-hostile players |
| `FRIEND_PC` | friendly players only |
| `NOT_FRIEND` | attackable targets: flagged players, war targets, monsters; **duel/olympiad opponents**. Without Ctrl, unflagged players are excluded |
| `NOT_FRIEND_PC` | attackable players only |
| `INVISIBLE` | only invisible creatures |
| `OBJECT_DEAD_NPC_BODY` | dead NPCs |
| `UNDEAD_REAL_ENEMY` | undead enemies |
| `WYVERN_OBJECT` | wyverns |

## Practical mapping for common skills

| Skill | targetType | affectScope | Behaviour |
|-------|-----------|-------------|-----------|
| Wind Strike / most nukes | `ENEMY` (`ENEMY_ONLY` at lvl 1) | `SINGLE` | one enemy, needs Ctrl vs unflagged players |
| Heal / Battle Heal | `TARGET` | `SINGLE` | selected friendly, self if self-targeted |
| Might, Shield, Wind Walk | `TARGET` | `SINGLE` | single-target buff |
| Group Heal / party songs | `MY_PARTY` or `TARGET` | `PARTY` | party members within `affectRange` |
| AoE nukes (Blazing Circle) | `ENEMY`/`GROUND` | `RANGE` | everyone hostile within `affectRange` of target/ground point |
| Self-centered AoE (Corpse Burst-like, aura attacks) | `SELF`/`NONE` | `POINT_BLANK` | enemies around the caster |
