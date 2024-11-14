mod auth_gs;
mod init_ls;
mod kick_player;
mod player_auth_response;
mod request_chars;

pub use self::{
    auth_gs::AuthGS, init_ls::InitLS, kick_player::KickPlayer,
    player_auth_response::PlayerAuthResponse, request_chars::RequestChars,
};

#[repr(i32)]
#[derive(Clone, Debug)]
pub enum PlayerKickedReasons {
    DataStealer = 0x01,
    GenericViolation = 0x08,
    SevenDaysSuspended = 0x10,
    PermanentlyBanned = 0x20,
}
