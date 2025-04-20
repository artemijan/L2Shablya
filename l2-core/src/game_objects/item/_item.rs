use crate::bitmask::BitMask;
use crate::game_objects::item::attribute::Attribute;
use anyhow::bail;
use entities::entities::item::Model;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ItemListType {
    AugmentBonus = 1,
    ElementalAttribute = 2,
    EnchantEffect = 4,
    VisualId = 8,
}

impl From<ItemListType> for u8 {
    fn from(item_list_type: ItemListType) -> u8 {
        item_list_type as u8
    }
}
impl From<ItemListType> for u32 {
    fn from(item_list_type: ItemListType) -> u32 {
        item_list_type as u32
    }
}
impl TryFrom<u8> for ItemListType
{
    type Error = anyhow::Error;
    fn try_from(item_list_type: u8) -> anyhow::Result<Self> {
        match item_list_type.into() {
            1 => Ok(ItemListType::AugmentBonus),
            2 => Ok(ItemListType::ElementalAttribute),
            4 => Ok(ItemListType::EnchantEffect),
            8 => Ok(ItemListType::VisualId),
            _ => bail!("Invalid item list type"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemObject {
    pub item_model: Model,
}

impl ItemObject {
    #[must_use]
    pub fn from_items(items: Vec<Model>) -> Vec<ItemObject> {
        items
            .into_iter()
            .map(|item| ItemObject { item_model: item })
            .collect()
    }
    #[must_use]
    pub fn get_attribute_attack(&self) -> (Attribute, u16) {
        //todo: implement me
        (Attribute::None, 0)
    }
    #[must_use]
    pub fn get_attribute_defence(&self) -> [(Attribute, u16); 3] {
        //todo: implement me
        [
            (Attribute::None, 0),
            (Attribute::None, 0),
            (Attribute::None, 0),
        ]
    }
    #[must_use]
    pub fn get_attribute_defence_total(&self) -> u16 {
        //todo: implement me
        self.get_attribute_defence().iter().map(|i| i.1).sum()
    }
    #[must_use]
    pub fn get_enchant_options(&self) -> &Vec<i32> {
        //todo: implement me
        static EMPTY_VEC: Vec<i32> = Vec::new();
        &EMPTY_VEC
    }
    #[must_use]
    pub fn get_visual_id(&self) -> i32 {
        //todo: implement me
        0
    }

    #[must_use]
    pub fn get_display_id(&self) -> i32 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_type_2(&self) -> u8 {
        // Item Type 2 : 00-weapon, 01-shield/armor, 02-ring/earring/necklace, 03-questitem, 04-adena, 05-item
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_custom_type_1(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_custom_type_2(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_time(&self) -> i32 {
        //todo: implement me
        //_time > 0 ? _time : _visualExpiration > 0 ? (int) _visualExpiration : -9999;
        -9999
    }
    #[must_use]
    pub fn is_quest_item(&self) -> bool {
        //todo: implement me
        false
    }
    #[must_use]
    pub fn is_available(&self) -> bool {
        //todo: implement me
        true
    }
    #[must_use]
    pub fn is_equipped(&self) -> bool {
        //todo: implement me
        false
    }
    #[must_use]
    pub fn get_location(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_body_part(&self) -> i64 {
        // Slot : 0006-lr.ear, 0008-neck, 0030-lr.finger, 0040-head, 0100-l.hand, 0200-gloves, 0400-chest, 0800-pants, 1000-feet, 4000-r.hand, 8000-r.hand
        //todo: implement me
        0
    }

    #[must_use]
    pub fn calculate_mask(&self) -> BitMask {
        //todo: implement me
        let mut mask = BitMask::new();
        if self.item_model.get_augmentation().is_some() {
            mask.add_mask(ItemListType::AugmentBonus);
        }
        if self.get_attribute_attack().1 > 0 || self.get_attribute_defence_total() > 0 {
            mask.add_mask(ItemListType::ElementalAttribute);
        }
        let options = self.get_enchant_options();
        for opt in options {
            if *opt > 0 {
                mask.add_mask(ItemListType::EnchantEffect);
                break;
            }
        }
        if self.get_visual_id() > 0 {
            mask.add_mask(ItemListType::VisualId);
        }
        mask
    }
}
