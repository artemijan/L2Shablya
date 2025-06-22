use l2_core::shared_packets::common::GSLoginFailReasons;
use strum::Display;

#[derive(Debug, Clone, Display, Eq, PartialEq, Hash, Copy, PartialOrd, Ord)]
pub enum GS {
    Initial,
    Connected,
    BfConnected,
    Authed,
}

impl GS {
    /// # Errors
    /// - transition not allowed
    pub fn transition_to(&mut self, desired_state: &GS) -> Result<(), GSLoginFailReasons> {
        match (&self, desired_state) {
            (Self::Initial, Self::Connected) => *self = Self::Connected,
            (Self::Connected, Self::BfConnected) => *self = Self::BfConnected,
            (Self::BfConnected, Self::Authed) => *self = Self::Authed,
            _ => {
                println!("Can't transition from connection state {self} to {desired_state}");
                return Err(GSLoginFailReasons::NotAuthed);
            }
        }
        Ok(())
    }
}
