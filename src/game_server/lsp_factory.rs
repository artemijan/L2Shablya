use super::handlers::LoginHandler;
use crate::common::packets::common::{GSLoginFail, HandleablePacket, ReadablePacket};
use crate::common::packets::gs_2_ls::ChangePassword;
use crate::common::packets::ls_2_gs::{
    AuthGS, InitLS, KickPlayer, PlayerAuthResponse, RequestChars,
};

pub fn build_ls_packet(
    data: &[u8],
) -> Option<Box<dyn HandleablePacket<HandlerType = LoginHandler>>> {
    if data.len() <= 1 {
        return None;
    }
    match data[0] {
        0x00 => Some(Box::new(InitLS::read(data)?)),
        0x01 => Some(Box::new(GSLoginFail::read(data)?)),
        0x02 => Some(Box::new(AuthGS::read(data)?)),
        0x03 => Some(Box::new(PlayerAuthResponse::read(data)?)),
        0x04 => Some(Box::new(KickPlayer::read(data)?)),
        0x05 => Some(Box::new(RequestChars::read(data)?)),
        0x06 => Some(Box::new(ChangePassword::read(data)?)),
        _ => {
            eprintln!("Unknown GS packet ID:0x{:02X}", data[0]);
            None
        }
    }
}
