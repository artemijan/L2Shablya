//todo: it's just a stub, we need to store it in DB

#[repr(u8)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy, PartialOrd, Ord)]
pub enum MacroType {
    None,
    Skill,
    Action,
    Text,
    Shortcut,
    Item,
    Delay,
}
impl From<u8> for MacroType {
    fn from(value: u8) -> Self {
        match value {
            1 => MacroType::Skill,
            2 => MacroType::Action,
            3 => MacroType::Text,
            4 => MacroType::Shortcut,
            5 => MacroType::Item,
            6 => MacroType::Delay,
            _ => MacroType::None,
        }
    }
}
impl From<MacroType> for u8 {
    fn from(value: MacroType) -> Self {
        match value {
            MacroType::Skill => 1,
            MacroType::Action => 2,
            MacroType::Text => 3,
            MacroType::Shortcut => 4,
            MacroType::Item => 5,
            MacroType::Delay => 6,
            _ => 0,
        }
    }
}
#[derive(Debug, Clone)]
pub struct MacroCommand(String);

impl MacroCommand {
    /// Gets the skill ID, item ID, page ID, depending on the marco use
    #[must_use]
    pub fn get_d1(&self) -> i32 {
        //todo: implement me
        0i32
    }
    /// Gets the skill level, shortcut ID, depending on the marco use.
    #[must_use]
    pub fn get_d2(&self) -> i32 {
        //todo: implement me
        0i32
    }
    #[must_use]
    pub fn get_cmd_name(&self) -> &str {
        //todo: implement me
        &self.0
    }
    #[must_use]
    pub fn get_type(&self) -> MacroType {
        //todo: implement me
        if self.0.starts_with("/delay") {
            MacroType::Delay
        } else if self.0.starts_with("/useskill") {
            MacroType::Skill
        } else {
            MacroType::None
        }
    }
}
#[derive(Debug, Clone)]
pub struct PlayerMacro {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub acronym: String,
    pub icon: i32,
    pub commands: Vec<MacroCommand>,
}
