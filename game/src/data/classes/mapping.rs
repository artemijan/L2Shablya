use anyhow::bail;
use entities::dao::char_info::Race;
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

impl Class {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn get_class(self) -> CharClass {
        #[allow(clippy::enum_glob_use)]
        use Class::*;
        #[allow(clippy::match_same_arms)]
        match self {
            Fighter => CharClass::fighter(self, Race::Human, None),
            Warrior => CharClass::fighter(self, Race::Human, Some(Fighter)),
            Gladiator => CharClass::fighter(self, Race::Human, Some(Warrior)),
            Warlord => CharClass::fighter(self, Race::Human, Some(Warrior)),
            Knight => CharClass::fighter(self, Race::Human, Some(Fighter)),
            Paladin => CharClass::fighter(self, Race::Human, Some(Knight)),
            DarkAvenger => CharClass::fighter(self, Race::Human, Some(Knight)),
            Rogue => CharClass::fighter(self, Race::Human, Some(Fighter)),
            TreasureHunter => CharClass::fighter(self, Race::Human, Some(Rogue)),
            Hawkeye => CharClass::fighter(self, Race::Human, Some(Rogue)),
            Mage => CharClass::mage(self, Race::Human, None),
            Wizard => CharClass::mage(self, Race::Human, Some(Mage)),
            Sorcerer => CharClass::mage(self, Race::Human, Some(Wizard)),
            Necromancer => CharClass::mage(self, Race::Human, Some(Wizard)),
            Warlock => CharClass::summoner(self, Race::Human, Some(Wizard)),
            Cleric => CharClass::mage(self, Race::Human, Some(Mage)),
            Bishop => CharClass::mage(self, Race::Human, Some(Cleric)),
            Prophet => CharClass::mage(self, Race::Human, Some(Cleric)),
            ElvenFighter => CharClass::fighter(self, Race::Elf, None),
            ElvenKnight => CharClass::fighter(self, Race::Elf, Some(ElvenFighter)),
            TempleKnight => CharClass::fighter(self, Race::Elf, Some(ElvenKnight)),
            Swordsinger => CharClass::fighter(self, Race::Elf, Some(ElvenKnight)),
            ElvenScout => CharClass::fighter(self, Race::Elf, Some(ElvenFighter)),
            PlainsWalker => CharClass::fighter(self, Race::Elf, Some(ElvenScout)),
            SilverRanger => CharClass::fighter(self, Race::Elf, Some(ElvenScout)),
            ElvenMage => CharClass::mage(self, Race::Elf, None),
            ElvenWizard => CharClass::mage(self, Race::Elf, Some(ElvenMage)),
            Spellsinger => CharClass::mage(self, Race::Elf, Some(ElvenWizard)),
            ElementalSummoner => CharClass::summoner(self, Race::Elf, Some(ElvenWizard)),
            Oracle => CharClass::mage(self, Race::Elf, Some(ElvenMage)),
            Elder => CharClass::mage(self, Race::Elf, Some(Oracle)),
            DarkFighter => CharClass::fighter(self, Race::DarkElf, None),
            PalusKnight => CharClass::fighter(self, Race::DarkElf, Some(DarkFighter)),
            ShillienKnight => CharClass::fighter(self, Race::DarkElf, Some(PalusKnight)),
            Bladedancer => CharClass::fighter(self, Race::DarkElf, Some(PalusKnight)),
            Assassin => CharClass::fighter(self, Race::DarkElf, Some(DarkFighter)),
            AbyssWalker => CharClass::fighter(self, Race::DarkElf, None),
            PhantomRanger => CharClass::fighter(self, Race::DarkElf, Some(Assassin)),
            DarkMage => CharClass::mage(self, Race::DarkElf, None),
            DarkWizard => CharClass::mage(self, Race::DarkElf, Some(DarkMage)),
            Spellhowler => CharClass::mage(self, Race::DarkElf, Some(DarkWizard)),
            PhantomSummoner => CharClass::summoner(self, Race::DarkElf, Some(DarkWizard)),
            ShillienOracle => CharClass::mage(self, Race::DarkElf, Some(DarkMage)),
            ShillienElder => CharClass::mage(self, Race::DarkElf, Some(ShillienOracle)),
            OrcFighter => CharClass::fighter(self, Race::Orc, None),
            OrcRaider => CharClass::fighter(self, Race::Orc, Some(OrcFighter)),
            Destroyer => CharClass::fighter(self, Race::Orc, Some(OrcRaider)),
            OrcMonk => CharClass::fighter(self, Race::Orc, Some(OrcFighter)),
            Tyrant => CharClass::fighter(self, Race::Orc, Some(OrcMonk)),
            OrcMage => CharClass::mage(self, Race::Orc, None),
            OrcShaman => CharClass::mage(self, Race::Orc, Some(OrcMage)),
            Overlord => CharClass::mage(self, Race::Orc, Some(OrcShaman)),
            Warcryer => CharClass::mage(self, Race::Orc, Some(OrcShaman)),
            DwarvenFighter => CharClass::fighter(self, Race::Dwarf, None),
            Scavenger => CharClass::fighter(self, Race::Dwarf, Some(DwarvenFighter)),
            BountyHunter => CharClass::fighter(self, Race::Dwarf, Some(Scavenger)),
            Artisan => CharClass::fighter(self, Race::Dwarf, Some(DwarvenFighter)),
            Warsmith => CharClass::fighter(self, Race::Dwarf, Some(Artisan)),
            Duelist => CharClass::fighter(self, Race::Human, Some(Gladiator)),
            Dreadnought => CharClass::fighter(self, Race::Human, Some(Warlord)),
            PhoenixKnight => CharClass::fighter(self, Race::Human, Some(Paladin)),
            HellKnight => CharClass::fighter(self, Race::Human, Some(DarkAvenger)),
            Sagittarius => CharClass::fighter(self, Race::Human, Some(Hawkeye)),
            Adventurer => CharClass::fighter(self, Race::Human, Some(TreasureHunter)),
            Archmage => CharClass::mage(self, Race::Human, Some(Sorcerer)),
            Soultaker => CharClass::mage(self, Race::Human, Some(Necromancer)),
            ArcanaLord => CharClass::summoner(self, Race::Human, Some(Warlock)),
            Cardinal => CharClass::mage(self, Race::Human, Some(Bishop)),
            Hierophant => CharClass::mage(self, Race::Human, Some(Prophet)),
            EvaTemplar => CharClass::fighter(self, Race::Elf, Some(TempleKnight)),
            SwordMuse => CharClass::fighter(self, Race::Elf, Some(Swordsinger)),
            WindRider => CharClass::fighter(self, Race::Elf, Some(PlainsWalker)),
            MoonlightSentinel => CharClass::fighter(self, Race::Elf, Some(SilverRanger)),
            MysticMuse => CharClass::mage(self, Race::Elf, Some(Spellsinger)),
            ElementalMaster => CharClass::summoner(self, Race::Elf, Some(ElementalSummoner)),
            EvaSaint => CharClass::mage(self, Race::Elf, Some(Elder)),
            ShillienTemplar => CharClass::fighter(self, Race::DarkElf, Some(ShillienKnight)),
            SpectralDancer => CharClass::fighter(self, Race::DarkElf, Some(Bladedancer)),
            GhostHunter => CharClass::fighter(self, Race::DarkElf, Some(AbyssWalker)),
            GhostSentinel => CharClass::fighter(self, Race::DarkElf, Some(PhantomRanger)),
            StormScreamer => CharClass::mage(self, Race::DarkElf, Some(Spellhowler)),
            SpectralMaster => CharClass::summoner(self, Race::DarkElf, Some(PhantomSummoner)),
            ShillienSaint => CharClass::mage(self, Race::DarkElf, Some(ShillienElder)),
            Titan => CharClass::fighter(self, Race::Orc, Some(Destroyer)),
            GrandKhavatari => CharClass::fighter(self, Race::Orc, Some(Tyrant)),
            Dominator => CharClass::mage(self, Race::Orc, Some(Overlord)),
            Doomcryer => CharClass::mage(self, Race::Orc, Some(Warcryer)),
            FortuneSeeker => CharClass::fighter(self, Race::Dwarf, Some(BountyHunter)),
            Maestro => CharClass::fighter(self, Race::Dwarf, Some(Warsmith)),
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
            Class::try_from(i).unwrap_or_else(|_| panic!("Invalid class ID: {i}"));
        }
    }
}
