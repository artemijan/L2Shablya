use anyhow::bail;
use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use l2_core::shared_packets::common::{GSLoginFail, ReadablePacket};
use l2_core::shared_packets::ls_2_gs::{AuthGS, InitLS, KickPlayer, PlayerAuthResponse, RequestChars};

pub fn build_ls_packet(
    data: &[u8],
) -> anyhow::Result<Box<dyn HandleablePacket<HandlerType = LoginHandler>>> {
    if data.is_empty() {
        bail!("Empty packet");
    }
    match data[0] {
        InitLS::PACKET_ID  => Ok(Box::new(InitLS::read(data)?)),
        GSLoginFail::PACKET_ID => Ok(Box::new(GSLoginFail::read(data)?)),
        AuthGS::PACKET_ID => Ok(Box::new(AuthGS::read(data)?)),
        PlayerAuthResponse::PACKET_ID => Ok(Box::new(PlayerAuthResponse::read(data)?)),
        KickPlayer::PACKET_ID => Ok(Box::new(KickPlayer::read(data)?)),
        RequestChars::PACKET_ID => Ok(Box::new(RequestChars::read(data)?)),
        _ => {
            bail!("Unknown GS packet ID:0x{:02X}", data[0]);
        }
    }
}
