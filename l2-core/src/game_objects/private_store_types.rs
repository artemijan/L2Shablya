#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PrivateStoreType {
    None = 0,
    Sell = 1,
    SellManage = 2,
    Buy = 3,
    BuyManage = 4,
    Manufacture = 5,
    PackageSell = 8,
    SellBuffs = 9,
}

impl PrivateStoreType {
    #[must_use]
    pub fn id(self) -> u8 {
        self as u8
    }
    #[must_use]
    pub fn get_by_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::None),
            1 => Some(Self::Sell),
            2 => Some(Self::SellManage),
            3 => Some(Self::Buy),
            4 => Some(Self::BuyManage),
            5 => Some(Self::Manufacture),
            8 => Some(Self::PackageSell),
            9 => Some(Self::SellBuffs),
            _ => None,
        }
    }
}