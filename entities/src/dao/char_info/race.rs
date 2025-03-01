#[repr(i32)]
#[derive(Clone, Debug, Copy, Hash, PartialOrd, Ord, Eq, PartialEq)]
pub enum Race {
    Human,
    Elf,
    DarkElf,
    Orc,
    Dwarf,
    Kamael,
    Ertheia,
    Animal,
    Beast,
    Bug,
    CastleGuard,
    Construct,
    Demonic,
    Divine,
    Dragon,
    Elemental,
    Etc,
    Fairy,
    Giant,
    Humanoid,
    Mercenary,
    None,
    Plant,
    SiegeWeapon,
    Undead,
    Friend,
}

fn try_from(value: i32) -> anyhow::Result<Race> {
    #[allow(clippy::enum_glob_use)]
    use Race::*;
    use anyhow::bail;
    match value {
        0 => Ok(Human),
        1 => Ok(Elf),
        2 => Ok(DarkElf),
        3 => Ok(Orc),
        4 => Ok(Dwarf),
        5 => Ok(Kamael),
        6 => Ok(Ertheia),
        7 => Ok(Animal),
        8 => Ok(Beast),
        9 => Ok(Bug),
        10 => Ok(CastleGuard),
        11 => Ok(Construct),
        12 => Ok(Demonic),
        13 => Ok(Divine),
        14 => Ok(Dragon),
        15 => Ok(Elemental),
        16 => Ok(Etc),
        17 => Ok(Fairy),
        18 => Ok(Giant),
        19 => Ok(Humanoid),
        20 => Ok(Mercenary),
        21 => Ok(None),
        22 => Ok(Plant),
        23 => Ok(SiegeWeapon),
        24 => Ok(Undead),
        _ => bail!("Unknown race value: {}", value),
    }
}

impl TryFrom<i32> for Race {
    type Error = anyhow::Error;
    fn try_from(value: i32) -> anyhow::Result<Self> {
        try_from(value)
    }
}

impl TryFrom<i8> for Race {
    type Error = anyhow::Error;
    fn try_from(value: i8) -> anyhow::Result<Self> {
        try_from(value.into())
    }
}

impl TryFrom<u8> for Race {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> anyhow::Result<Self> {
        try_from(value.into())
    }
}