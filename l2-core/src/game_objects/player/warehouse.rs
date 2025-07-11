use crate::game_objects::item::ItemObject;
use entities::entities::item;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Warehouse {
    pub items: HashMap<i32, ItemObject>,
}

impl Warehouse {
    pub fn from_items(items: Vec<item::Model>) -> Self {
        Self {
            items: ItemObject::from_items(items),
        }
    }
    pub fn empty() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    #[must_use]
    pub fn get_limit(&self) -> u8 {
        //todo: implement me
        100
    }
    #[must_use]
    pub fn get_current_load(&self) -> i32 {
        0i32 //todo: implement me
    }
    #[must_use]
    pub fn get_max_load(&self) -> i32 {
        202_860 //todo: implement me
    }

    #[must_use]
    pub fn get_size(&self) -> u16 {
        0u16 //todo: implement me
    }
}
