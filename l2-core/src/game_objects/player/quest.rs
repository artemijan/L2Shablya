use entities::entities::quest;

#[derive(Debug, Clone)]
pub struct Quest {
    model: quest::Model,
}
impl Quest {
    #[must_use]
    pub fn get_id(&self) -> i32 {
        //todo: implement me
        -1
    }
    #[must_use]
    pub fn is_started(&self) -> bool {
        //todo: implement me
        false
    }
    #[must_use]
    pub fn is_completed(&self) -> bool {
        //todo: implement me
        false
    }
    
    #[must_use]
    pub fn get_condition_bit_set(&self) -> i32 {
        if self.is_started() {
            todo!()
        }
        0
    }
}
