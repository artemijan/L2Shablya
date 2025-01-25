#[repr(i32)]
#[allow(unused)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CharNameResponseVariant {
    Ok = -1,
    CharCreationFailed = 0,
    TooManyChars = 1,
    AlreadyExists = 2,
    InvalidLength = 3,
    InvalidName = 4,
    NotAllowed = 5,
    ChooseAnotherSvr = 6,
}
