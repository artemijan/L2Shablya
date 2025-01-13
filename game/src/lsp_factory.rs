use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use l2_core::shared_packets::common::{GSLoginFail, ReadablePacket};
use l2_core::shared_packets::ls_2_gs::{AuthGS, InitLS, KickPlayer, PlayerAuthResponse, RequestChars};
use tracing::error;

pub fn build_ls_packet(
    data: &[u8],
) -> Option<Box<dyn HandleablePacket<HandlerType = LoginHandler>>> {
    if data.is_empty() {
        return None;
    }
    match data[0] {
        InitLS::PACKET_ID => Some(Box::new(InitLS::read(data)?)),
        GSLoginFail::PACKET_ID => Some(Box::new(GSLoginFail::read(data)?)),
        AuthGS::PACKET_ID => Some(Box::new(AuthGS::read(data)?)),
        PlayerAuthResponse::PACKET_ID => Some(Box::new(PlayerAuthResponse::read(data)?)),
        KickPlayer::PACKET_ID => Some(Box::new(KickPlayer::read(data)?)),
        RequestChars::PACKET_ID => Some(Box::new(RequestChars::read(data)?)),
        _ => {
            error!("Unknown GS packet ID:0x{:02X}", data[0]);
            None
        }
    }
}
