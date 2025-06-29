use crate::bitmask::BitMask;
use sea_orm::strum::IntoEnumIterator;
use sea_orm::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
#[repr(u32)]
pub enum UserInfoType {
    Relation = 0x00,
    BasicInfo = 0x01,
    BaseStats = 0x02,
    MaxHpCpMp = 0x03,
    CurrentHpMpCpExpSp = 0x04,
    EnchantLevel = 0x05,
    Appearance = 0x06,
    Status = 0x07,
    Stats = 0x08,
    Elementals = 0x09,
    Position = 0x0A,
    Speed = 0x0B,
    Multiplier = 0x0C,
    ColRadiusHeight = 0x0D,
    AtkElemental = 0x0E,
    Clan = 0x0F,
    Social = 0x10,
    VitaFame = 0x11,
    Slots = 0x12,
    Movements = 0x13,
    Color = 0x14,
    InventoryLimit = 0x15,
    TrueHero = 0x16,
}

impl From<UserInfoType> for u32 {
    fn from(value: UserInfoType) -> Self {
        value as u32
    }
}

impl UserInfoType {

    #[must_use]
    pub fn all() -> BitMask {
        let mut bm = BitMask::new(24);
        for v in UserInfoType::iter() {
            bm.add_mask(v);
        }
        bm
    }
    #[must_use]
    pub fn calculate_block_size(bit_mask: &BitMask) -> u32 {
        let mut size = 0;
        for v in UserInfoType::iter() {
            if bit_mask.contains_mask(v) {
                size += v.block_size();
            }
        }
        size
    }
    #[must_use]
    pub fn block_size(self) -> u32 {
        match self {
            UserInfoType::Relation | UserInfoType::EnchantLevel | UserInfoType::Movements => 4,
            UserInfoType::BasicInfo => 16,
            UserInfoType::BaseStats
            | UserInfoType::Multiplier
            | UserInfoType::Position
            | UserInfoType::ColRadiusHeight
            | UserInfoType::Speed => 18,
            UserInfoType::MaxHpCpMp | UserInfoType::Elementals => 14,
            UserInfoType::CurrentHpMpCpExpSp => 38,
            UserInfoType::Appearance | UserInfoType::VitaFame => 15,
            UserInfoType::Status => 6,
            UserInfoType::Stats => 56,
            UserInfoType::AtkElemental => 5,
            UserInfoType::Clan => 32,
            UserInfoType::Social => 22,
            UserInfoType::Slots | UserInfoType::TrueHero | UserInfoType::InventoryLimit => 9,
            UserInfoType::Color => 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bitmask::BitMask;
    use crate::game_objects::player::user_info::UserInfoType;
    #[test]
    fn test_mask() {
        let mask = UserInfoType::all();
        assert!(mask.contains_mask(UserInfoType::Relation as u32));
        assert!(mask.contains_mask(UserInfoType::Appearance as u32));
    }
    #[test]
    fn test_mask_does_not_contain() {
        let mut mask = BitMask::new(24);
        mask.add_mask(UserInfoType::BasicInfo as u32);
        assert!(mask.contains_mask(UserInfoType::BasicInfo as u32));
        assert!(!mask.contains_mask(UserInfoType::Appearance as u32));
    }
    #[test]
    fn test_mask_calculate_block_size() {
        let mut mask = BitMask::new(24);
        mask.add_mask(UserInfoType::BasicInfo as u32);
        mask.add_mask(UserInfoType::Clan as u32);
        let block_size = UserInfoType::calculate_block_size(&mask);
        assert_eq!(block_size, 16 + 32);
    }
}
