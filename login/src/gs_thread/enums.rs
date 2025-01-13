use l2_core::shared_packets::common::GSLoginFailReasons;
use strum::Display;

#[derive(Debug, Clone, Display)]
pub enum GS {
    Initial,
    Connected,
    BfConnected,
    Authed,
}

impl GS {
    pub fn transition_to(&mut self, desired_state: &GS) -> Result<(), GSLoginFailReasons> {
        match (&self, desired_state) {
            (Self::Initial, Self::Connected) => *self = Self::Connected,
            (Self::Connected, Self::BfConnected) => *self = Self::BfConnected,
            (Self::BfConnected, Self::Authed) => *self = Self::Authed,
            _ => {
                return Err(GSLoginFailReasons::NotAuthed);
            }
        }
        Ok(())
    }
}
