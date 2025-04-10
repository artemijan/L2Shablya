#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperDoll {
    Under = 0,
    Head = 1,
    Hair = 2,
    Hair2 = 3,
    Neck = 4,
    RHand = 5,
    Chest = 6,
    LHand = 7,
    Rear = 8,
    Lear = 9,
    Gloves = 10,
    Legs = 11,
    Feet = 12,
    RFinger = 13,
    LFinger = 14,
    LBracelet = 15,
    RBracelet = 16,
    Deco1 = 17,
    Deco2 = 18,
    Deco3 = 19,
    Deco4 = 20,
    Deco5 = 21,
    Deco6 = 22,
    Cloak = 23,
    Belt = 24,
    Brooch = 25,
    BroochJewel1 = 26,
    BroochJewel2 = 27,
    BroochJewel3 = 28,
    BroochJewel4 = 29,
    BroochJewel5 = 30,
    BroochJewel6 = 31,
    TotalSlots = 32,
}

impl PaperDoll {
    #[must_use]
    pub fn ordered_ids() -> [PaperDoll; 33] {
        [
            Self::Under,
            Self::Rear,
            Self::Lear,
            Self::Neck,
            Self::RFinger,
            Self::LFinger,
            Self::Head,
            Self::RHand,
            Self::LHand,
            Self::Gloves,
            Self::Chest,
            Self::Legs,
            Self::Feet,
            Self::Cloak,
            Self::RHand, // I don't give a fuck why Rhand declared twice, copied as is from L2j
            Self::Hair,
            Self::Hair2,
            Self::RBracelet,
            Self::LBracelet,
            Self::Deco1,
            Self::Deco2,
            Self::Deco3,
            Self::Deco4,
            Self::Deco5,
            Self::Deco6,
            Self::Belt,
            Self::Brooch,
            Self::BroochJewel1,
            Self::BroochJewel2,
            Self::BroochJewel3,
            Self::BroochJewel4,
            Self::BroochJewel5,
            Self::BroochJewel6,
        ]
    }

    #[must_use]
    pub fn visual_ids() -> [PaperDoll; 9] {
        [
            Self::RHand,
            Self::LHand,
            Self::Gloves,
            Self::Chest,
            Self::Legs,
            Self::Feet,
            Self::RHand, // I don't give a fuck why Rhand declared twice, copied as is from L2j
            Self::Hair,
            Self::Hair2,
        ]
    }
}