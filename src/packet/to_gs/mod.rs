mod auth_gs;
mod init_ls;
mod player_auth_response;
mod request_chars;
mod kick_player;

pub use self::{auth_gs::AuthGS, init_ls::InitLS, player_auth_response::PlayerAuthResponse, request_chars::RequestChars, kick_player::KickPlayer};

#[repr(i32)]
#[derive(Clone, Debug)]
pub enum PlayerKickedReasons {
    DataStealer = 0x01,
    GenericViolation = 0x08,
    SevenDaysSuspended = 0x10,
    PermanentlyBanned = 0x20,
}