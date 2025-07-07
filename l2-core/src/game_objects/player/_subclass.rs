#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubclassType{
    BaseClass,
    DualClass,
    Subclass
}
#[derive(Debug, Clone)]
pub struct Subclass{
    pub index:i32,
    pub class_id:i8,
    pub level: u8,
    pub class_type: SubclassType
}

impl From<SubclassType> for u8{
    fn from(value: SubclassType) -> Self {
        value as u8
    }
}