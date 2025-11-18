use std::ops::{BitOr, BitOrAssign};

#[derive(Eq, PartialEq, Debug, Clone, Copy, Hash)]
#[repr(u32)]
pub enum RelationChanges {
    Party1 = 0x1,
    Party2 = 0x2,
    Party3 = 0x4,
    Party4 = 0x8,
    PartyLeader = 0x10,
    HasParty = 0x20,
    ClanMember = 0x40,
    Leader = 0x80,
    ClanMate = 0x100,
    InSiege = 0x200,
    Attacker = 0x400,
    Ally = 0x800,
    Enemy = 0x1000,
    DeclaredWar = 0x4000,
    MutualWar = 0x8000,
    AllyMember = 0x10000,
    TerritoryWar = 0x80000,
}
impl BitOr<u32> for RelationChanges {
    type Output = u32;

    fn bitor(self, rhs: u32) -> u32 {
        (self as u32) | rhs
    }
}

impl BitOrAssign<RelationChanges> for u32 {
    fn bitor_assign(&mut self, rhs: RelationChanges) {
        *self |= rhs as u32;
    }
}
impl RelationChanges {
    pub fn party_index_mask(index: u32) -> u32 {
        if index > 8 {
            panic!("Invalid party index");
        }
        match index {
            0 => Self::PartyLeader as u32, //0x10
            _ => 9 - index,
        }
    }
}
