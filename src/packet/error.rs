use crate::packet::common::SendablePacket;
use std::fmt;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub struct PacketRunError {
    pub msg: Option<String>,
    pub response: Option<Box<dyn SendablePacket>>,
}

impl fmt::Display for PacketRunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.msg)
    }
}
