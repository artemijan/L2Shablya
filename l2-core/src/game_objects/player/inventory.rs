#[derive(Debug, Clone)]
pub struct Inventory;
impl Inventory {
    #[must_use]
    pub fn get_talisman_slots(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_brooch_jewel_slots(&self) -> u8 {
        //todo: implement me
        0
    }
    #[must_use]
    pub fn get_limit(&self) -> u8 {
        //todo: implement me
        200
    }

}
