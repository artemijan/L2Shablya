#[repr(i32)]
#[derive(Clone, Debug)]
pub enum CharDeletionFailReasons {
    None,
    Unknown,
    PledgeMember,
    PledgeMaster,
    ProhibitCharDeletion,
    Commission,
    Mentor,
    Mentee,
    Mail,
}
