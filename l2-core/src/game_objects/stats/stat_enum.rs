use strum::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum Stat {
    // HP, MP & CP
    MaxHp,
    MaxMp,
    MaxCp,
    RegenHp,
    RegenMp,
    RegenCp,

    // Basic Stats
    PAtk,
    MAtk,
    PDef,
    MDef,
    PAtkSpd,
    MAtkSpd,
    PAccuracy,
    MAccuracy,
    PEvasion,
    MEvasion,
    PCriticalRate,
    MCriticalRate,
    PCriticalDamage,
    MCriticalDamage,

    // Base Stats
    Str,
    Int,
    Dex,
    Wit,
    Con,
    Men,

    // Shield
    ShieldDef,
    ShieldRate,

    // Misc
    MoveSpeed,
    AttackRange,
    RandomDamage,

    // Multipliers
    PvpPhysDmg,
    PvpMagicalDmg,
    PvpPhysSkillsDmg,
    PvePhysDmg,
    PveMagicalDmg,
}
