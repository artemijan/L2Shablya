use std::fmt;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub struct PacketRun {
    pub msg: Option<String>,
}
impl From<anyhow::Error> for PacketRun {
    fn from(error: anyhow::Error) -> Self {
        PacketRun {
            msg: Some(error.to_string()), // Use the error's string representation
        }
    }
}

impl fmt::Display for PacketRun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.msg)
    }
}
