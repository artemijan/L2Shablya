pub mod mapping;

#[repr(i32)]
#[derive(Clone, Debug, Copy)]
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
