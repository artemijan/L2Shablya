use anyhow::bail;
use crate::game_objects::race::Race;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub struct CharClass {
    pub id: Class,
    pub is_mage: bool,
    pub is_summoner: bool,
    pub parent: Option<Class>,
    pub race: Race,
}

impl CharClass {
    #[must_use]
    pub const fn new(
        id: Class,
        is_mage: bool,
        is_summoner: bool,
        race: Race,
        parent: Option<Class>,
    ) -> Self {
        Self {
            id,
            is_mage,
            is_summoner,
            parent,
            race,
        }
    }
    #[must_use]
    pub const fn mage(id: Class, race: Race, parent: Option<Class>) -> Self {
        Self {
            id,
            is_mage: true,
            race,
            is_summoner: false,
            parent,
        }
    }
    #[must_use]
    pub const fn summoner(id: Class, race: Race, parent: Option<Class>) -> Self {
        Self {
            id,
            is_mage: true,
            race,
            is_summoner: true,
            parent,
        }
    }
    #[must_use]
    pub const fn fighter(id: Class, race: Race, parent: Option<Class>) -> Self {
        Self {
            id,
            is_mage: false,
            race,
            is_summoner: false,
            parent,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Debug, Copy, Hash, PartialOrd, Ord, Eq, PartialEq)]
pub enum Class {
    Fighter = 0,
    Warrior = 1,
    Gladiator = 2,
    Warlord = 3,
    Knight = 4,
    Paladin = 5,
    DarkAvenger = 6,
    Rogue = 7,
    TreasureHunter = 8,
    Hawkeye = 9,
    Mage = 10,
    Wizard = 11,
    Sorcerer = 12,
    Necromancer = 13,
    Warlock = 14,
    Cleric = 15,
    Bishop = 16,
    Prophet = 17,
    ElvenFighter = 18,
    ElvenKnight = 19,
    TempleKnight = 20,
    Swordsinger = 21,
    ElvenScout = 22,
    PlainsWalker = 23,
    SilverRanger = 24,
    ElvenMage = 25,
    ElvenWizard = 26,
    Spellsinger = 27,
    ElementalSummoner = 28,
    Oracle = 29,
    Elder = 30,
    DarkFighter = 31,
    PalusKnight = 32,
    ShillienKnight = 33,
    Bladedancer = 34,
    Assassin = 35,
    AbyssWalker = 36,
    PhantomRanger = 37,
    DarkMage = 38,
    DarkWizard = 39,
    Spellhowler = 40,
    PhantomSummoner = 41,
    ShillienOracle = 42,
    ShillienElder = 43,
    OrcFighter = 44,
    OrcRaider = 45,
    Destroyer = 46,
    OrcMonk = 47,
    Tyrant = 48,
    OrcMage = 49,
    OrcShaman = 50,
    Overlord = 51,
    Warcryer = 52,
    DwarvenFighter = 53,
    Scavenger = 54,
    BountyHunter = 55,
    Artisan = 56,
    Warsmith = 57,
    Duelist = 88,
    Dreadnought = 89,
    PhoenixKnight = 90,
    HellKnight = 91,
    Sagittarius = 92,
    Adventurer = 93,
    Archmage = 94,
    Soultaker = 95,
    ArcanaLord = 96,
    Cardinal = 97,
    Hierophant = 98,
    EvaTemplar = 99,
    SwordMuse = 100,
    WindRider = 101,
    MoonlightSentinel = 102,
    MysticMuse = 103,
    ElementalMaster = 104,
    EvaSaint = 105,
    ShillienTemplar = 106,
    SpectralDancer = 107,
    GhostHunter = 108,
    GhostSentinel = 109,
    StormScreamer = 110,
    SpectralMaster = 111,
    ShillienSaint = 112,
    Titan = 113,
    GrandKhavatari = 114,
    Dominator = 115,
    Doomcryer = 116,
    FortuneSeeker = 117,
    Maestro = 118,
}
#[allow(clippy::from_over_into)]
impl Into<u8> for Class {
    fn into(self) -> u8 {
        self as u8
    }
}

#[allow(clippy::from_over_into)]
impl Into<u16> for Class {
    fn into(self) -> u16 {
        self as u16
    }
}
#[allow(clippy::from_over_into)]
impl Into<u32> for Class {
    fn into(self) -> u32 {
        self as u32
    }
}
#[allow(clippy::from_over_into)]
impl Into<u64> for Class {
    fn into(self) -> u64 {
        self as u64
    }
}
impl TryFrom<u8> for Class {
    type Error = anyhow::Error;
    #[allow(clippy::too_many_lines)]
    fn try_from(value: u8) -> anyhow::Result<Self> {
        #[allow(clippy::enum_glob_use)]
        use Class::*;
        match value {
            0 => Ok(Fighter),
            1 => Ok(Warrior),
            2 => Ok(Gladiator),
            3 => Ok(Warlord),
            4 => Ok(Knight),
            5 => Ok(Paladin),
            6 => Ok(DarkAvenger),
            7 => Ok(Rogue),
            8 => Ok(TreasureHunter),
            9 => Ok(Hawkeye),
            10 => Ok(Mage),
            11 => Ok(Wizard),
            12 => Ok(Sorcerer),
            13 => Ok(Necromancer),
            14 => Ok(Warlock),
            15 => Ok(Cleric),
            16 => Ok(Bishop),
            17 => Ok(Prophet),
            18 => Ok(ElvenFighter),
            19 => Ok(ElvenKnight),
            20 => Ok(TempleKnight),
            21 => Ok(Swordsinger),
            22 => Ok(ElvenScout),
            23 => Ok(PlainsWalker),
            24 => Ok(SilverRanger),
            25 => Ok(ElvenMage),
            26 => Ok(ElvenWizard),
            27 => Ok(Spellsinger),
            28 => Ok(ElementalSummoner),
            29 => Ok(Oracle),
            30 => Ok(Elder),
            31 => Ok(DarkFighter),
            32 => Ok(PalusKnight),
            33 => Ok(ShillienKnight),
            34 => Ok(Bladedancer),
            35 => Ok(Assassin),
            36 => Ok(AbyssWalker),
            37 => Ok(PhantomRanger),
            38 => Ok(DarkMage),
            39 => Ok(DarkWizard),
            40 => Ok(Spellhowler),
            41 => Ok(PhantomSummoner),
            42 => Ok(ShillienOracle),
            43 => Ok(ShillienElder),
            44 => Ok(OrcFighter),
            45 => Ok(OrcRaider),
            46 => Ok(Destroyer),
            47 => Ok(OrcMonk),
            48 => Ok(Tyrant),
            49 => Ok(OrcMage),
            50 => Ok(OrcShaman),
            51 => Ok(Overlord),
            52 => Ok(Warcryer),
            53 => Ok(DwarvenFighter),
            54 => Ok(Scavenger),
            55 => Ok(BountyHunter),
            56 => Ok(Artisan),
            57 => Ok(Warsmith),
            88 => Ok(Duelist),
            89 => Ok(Dreadnought),
            90 => Ok(PhoenixKnight),
            91 => Ok(HellKnight),
            92 => Ok(Sagittarius),
            93 => Ok(Adventurer),
            94 => Ok(Archmage),
            95 => Ok(Soultaker),
            96 => Ok(ArcanaLord),
            97 => Ok(Cardinal),
            98 => Ok(Hierophant),
            99 => Ok(EvaTemplar),
            100 => Ok(SwordMuse),
            101 => Ok(WindRider),
            102 => Ok(MoonlightSentinel),
            103 => Ok(MysticMuse),
            104 => Ok(ElementalMaster),
            105 => Ok(EvaSaint),
            106 => Ok(ShillienTemplar),
            107 => Ok(SpectralDancer),
            108 => Ok(GhostHunter),
            109 => Ok(GhostSentinel),
            110 => Ok(StormScreamer),
            111 => Ok(SpectralMaster),
            112 => Ok(ShillienSaint),
            113 => Ok(Titan),
            114 => Ok(GrandKhavatari),
            115 => Ok(Dominator),
            116 => Ok(Doomcryer),
            117 => Ok(FortuneSeeker),
            118 => Ok(Maestro),
            _ => bail!("Invalid class ID"),
        }
    }
}

impl TryFrom<i8> for Class {
    type Error = anyhow::Error;
    #[allow(clippy::cast_sign_loss)]
    fn try_from(value: i8) -> anyhow::Result<Self> {
        Class::try_from(value as u8)
    }
}

impl Class {
    #[must_use]
    pub fn get_root(&self) -> CharClass {
        if let Some(parent) = self.get_class().parent {
            parent.get_root()
        } else {
            self.get_class()
        }
    }
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn get_class(self) -> CharClass {
        #[allow(clippy::match_same_arms)]
        match self {
            Self::Fighter => CharClass::fighter(self, Race::Human, None),
            Self::Warrior => CharClass::fighter(self, Race::Human, Some(Self::Fighter)),
            Self::Gladiator => CharClass::fighter(self, Race::Human, Some(Self::Warrior)),
            Self::Warlord => CharClass::fighter(self, Race::Human, Some(Self::Warrior)),
            Self::Knight => CharClass::fighter(self, Race::Human, Some(Self::Fighter)),
            Self::Paladin => CharClass::fighter(self, Race::Human, Some(Self::Knight)),
            Self::DarkAvenger => CharClass::fighter(self, Race::Human, Some(Self::Knight)),
            Self::Rogue => CharClass::fighter(self, Race::Human, Some(Self::Fighter)),
            Self::TreasureHunter => CharClass::fighter(self, Race::Human, Some(Self::Rogue)),
            Self::Hawkeye => CharClass::fighter(self, Race::Human, Some(Self::Rogue)),
            Self::Mage => CharClass::mage(self, Race::Human, None),
            Self::Wizard => CharClass::mage(self, Race::Human, Some(Self::Mage)),
            Self::Sorcerer => CharClass::mage(self, Race::Human, Some(Self::Wizard)),
            Self::Necromancer => CharClass::mage(self, Race::Human, Some(Self::Wizard)),
            Self::Warlock => CharClass::summoner(self, Race::Human, Some(Self::Wizard)),
            Self::Cleric => CharClass::mage(self, Race::Human, Some(Self::Mage)),
            Self::Bishop => CharClass::mage(self, Race::Human, Some(Self::Cleric)),
            Self::Prophet => CharClass::mage(self, Race::Human, Some(Self::Cleric)),
            Self::ElvenFighter => CharClass::fighter(self, Race::Elf, None),
            Self::ElvenKnight => CharClass::fighter(self, Race::Elf, Some(Self::ElvenFighter)),
            Self::TempleKnight => CharClass::fighter(self, Race::Elf, Some(Self::ElvenKnight)),
            Self::Swordsinger => CharClass::fighter(self, Race::Elf, Some(Self::ElvenKnight)),
            Self::ElvenScout => CharClass::fighter(self, Race::Elf, Some(Self::ElvenFighter)),
            Self::PlainsWalker => CharClass::fighter(self, Race::Elf, Some(Self::ElvenScout)),
            Self::SilverRanger => CharClass::fighter(self, Race::Elf, Some(Self::ElvenScout)),
            Self::ElvenMage => CharClass::mage(self, Race::Elf, None),
            Self::ElvenWizard => CharClass::mage(self, Race::Elf, Some(Self::ElvenMage)),
            Self::Spellsinger => CharClass::mage(self, Race::Elf, Some(Self::ElvenWizard)),
            Self::ElementalSummoner => {
                CharClass::summoner(self, Race::Elf, Some(Self::ElvenWizard))
            }
            Self::Oracle => CharClass::mage(self, Race::Elf, Some(Self::ElvenMage)),
            Self::Elder => CharClass::mage(self, Race::Elf, Some(Self::Oracle)),
            Self::DarkFighter => CharClass::fighter(self, Race::DarkElf, None),
            Self::PalusKnight => CharClass::fighter(self, Race::DarkElf, Some(Self::DarkFighter)),
            Self::ShillienKnight => {
                CharClass::fighter(self, Race::DarkElf, Some(Self::PalusKnight))
            }
            Self::Bladedancer => CharClass::fighter(self, Race::DarkElf, Some(Self::PalusKnight)),
            Self::Assassin => CharClass::fighter(self, Race::DarkElf, Some(Self::DarkFighter)),
            Self::AbyssWalker => CharClass::fighter(self, Race::DarkElf, None),
            Self::PhantomRanger => CharClass::fighter(self, Race::DarkElf, Some(Self::Assassin)),
            Self::DarkMage => CharClass::mage(self, Race::DarkElf, None),
            Self::DarkWizard => CharClass::mage(self, Race::DarkElf, Some(Self::DarkMage)),
            Self::Spellhowler => CharClass::mage(self, Race::DarkElf, Some(Self::DarkWizard)),
            Self::PhantomSummoner => {
                CharClass::summoner(self, Race::DarkElf, Some(Self::DarkWizard))
            }
            Self::ShillienOracle => CharClass::mage(self, Race::DarkElf, Some(Self::DarkMage)),
            Self::ShillienElder => CharClass::mage(self, Race::DarkElf, Some(Self::ShillienOracle)),
            Self::OrcFighter => CharClass::fighter(self, Race::Orc, None),
            Self::OrcRaider => CharClass::fighter(self, Race::Orc, Some(Self::OrcFighter)),
            Self::Destroyer => CharClass::fighter(self, Race::Orc, Some(Self::OrcRaider)),
            Self::OrcMonk => CharClass::fighter(self, Race::Orc, Some(Self::OrcFighter)),
            Self::Tyrant => CharClass::fighter(self, Race::Orc, Some(Self::OrcMonk)),
            Self::OrcMage => CharClass::mage(self, Race::Orc, None),
            Self::OrcShaman => CharClass::mage(self, Race::Orc, Some(Self::OrcMage)),
            Self::Overlord => CharClass::mage(self, Race::Orc, Some(Self::OrcShaman)),
            Self::Warcryer => CharClass::mage(self, Race::Orc, Some(Self::OrcShaman)),
            Self::DwarvenFighter => CharClass::fighter(self, Race::Dwarf, None),
            Self::Scavenger => CharClass::fighter(self, Race::Dwarf, Some(Self::DwarvenFighter)),
            Self::BountyHunter => CharClass::fighter(self, Race::Dwarf, Some(Self::Scavenger)),
            Self::Artisan => CharClass::fighter(self, Race::Dwarf, Some(Self::DwarvenFighter)),
            Self::Warsmith => CharClass::fighter(self, Race::Dwarf, Some(Self::Artisan)),
            Self::Duelist => CharClass::fighter(self, Race::Human, Some(Self::Gladiator)),
            Self::Dreadnought => CharClass::fighter(self, Race::Human, Some(Self::Warlord)),
            Self::PhoenixKnight => CharClass::fighter(self, Race::Human, Some(Self::Paladin)),
            Self::HellKnight => CharClass::fighter(self, Race::Human, Some(Self::DarkAvenger)),
            Self::Sagittarius => CharClass::fighter(self, Race::Human, Some(Self::Hawkeye)),
            Self::Adventurer => CharClass::fighter(self, Race::Human, Some(Self::TreasureHunter)),
            Self::Archmage => CharClass::mage(self, Race::Human, Some(Self::Sorcerer)),
            Self::Soultaker => CharClass::mage(self, Race::Human, Some(Self::Necromancer)),
            Self::ArcanaLord => CharClass::summoner(self, Race::Human, Some(Self::Warlock)),
            Self::Cardinal => CharClass::mage(self, Race::Human, Some(Self::Bishop)),
            Self::Hierophant => CharClass::mage(self, Race::Human, Some(Self::Prophet)),
            Self::EvaTemplar => CharClass::fighter(self, Race::Elf, Some(Self::TempleKnight)),
            Self::SwordMuse => CharClass::fighter(self, Race::Elf, Some(Self::Swordsinger)),
            Self::WindRider => CharClass::fighter(self, Race::Elf, Some(Self::PlainsWalker)),
            Self::MoonlightSentinel => {
                CharClass::fighter(self, Race::Elf, Some(Self::SilverRanger))
            }
            Self::MysticMuse => CharClass::mage(self, Race::Elf, Some(Self::Spellsinger)),
            Self::ElementalMaster => {
                CharClass::summoner(self, Race::Elf, Some(Self::ElementalSummoner))
            }
            Self::EvaSaint => CharClass::mage(self, Race::Elf, Some(Self::Elder)),
            Self::ShillienTemplar => {
                CharClass::fighter(self, Race::DarkElf, Some(Self::ShillienKnight))
            }
            Self::SpectralDancer => {
                CharClass::fighter(self, Race::DarkElf, Some(Self::Bladedancer))
            }
            Self::GhostHunter => CharClass::fighter(self, Race::DarkElf, Some(Self::AbyssWalker)),
            Self::GhostSentinel => {
                CharClass::fighter(self, Race::DarkElf, Some(Self::PhantomRanger))
            }
            Self::StormScreamer => CharClass::mage(self, Race::DarkElf, Some(Self::Spellhowler)),
            Self::SpectralMaster => {
                CharClass::summoner(self, Race::DarkElf, Some(Self::PhantomSummoner))
            }
            Self::ShillienSaint => CharClass::mage(self, Race::DarkElf, Some(Self::ShillienElder)),
            Self::Titan => CharClass::fighter(self, Race::Orc, Some(Self::Destroyer)),
            Self::GrandKhavatari => CharClass::fighter(self, Race::Orc, Some(Self::Tyrant)),
            Self::Dominator => CharClass::mage(self, Race::Orc, Some(Self::Overlord)),
            Self::Doomcryer => CharClass::mage(self, Race::Orc, Some(Self::Warcryer)),
            Self::FortuneSeeker => CharClass::fighter(self, Race::Dwarf, Some(Self::BountyHunter)),
            Self::Maestro => CharClass::fighter(self, Race::Dwarf, Some(Self::Warsmith)),
        }
    }
}
// Implement Deserialize for ClassId
impl<'de> Deserialize<'de> for Class {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Class::try_from(value).map_err(|_| {
            serde::de::Error::custom(format!("Invalid class ID: {value}, it is not implemented."))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_all_classes() {
        for i in 1..=118u8 {
            if (58..88).contains(&i) {
                continue;
            }
            let cls = Class::try_from(i).unwrap_or_else(|_| panic!("Invalid class ID: {i}"));
            let _ = cls.get_class();
        }
    }
}
