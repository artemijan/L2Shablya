use anyhow::bail;
use bytes::BytesMut;
use l2_core::shared_packets::common::{GSLoginFail, ReadablePacket};
use l2_core::shared_packets::ls_2_gs::{
    AuthGS, InitLS, KickPlayer, PlayerAuthResponse, RequestChars,
};
use macro_common::PacketEnum;
use strum::Display;

#[derive(Clone, Debug, Display, PacketEnum)]
pub enum LSPackets {
    InitLS(InitLS),
    GSLoginFail(GSLoginFail),
    AuthGS(AuthGS),
    PlayerAuthResponse(PlayerAuthResponse),
    KickPlayer(KickPlayer),
    RequestChars(RequestChars),
}

pub fn build_ls_packet(data: BytesMut) -> anyhow::Result<LSPackets> {
    if data.len() < 2 {
        bail!("Not enough data to build packet: {data:?}");
    }
    match data[0] {
        InitLS::PACKET_ID => Ok(LSPackets::InitLS(InitLS::read(data)?)),
        GSLoginFail::PACKET_ID => Ok(LSPackets::GSLoginFail(GSLoginFail::read(data)?)),
        AuthGS::PACKET_ID => Ok(LSPackets::AuthGS(AuthGS::read(data)?)),
        PlayerAuthResponse::PACKET_ID => Ok(LSPackets::PlayerAuthResponse(
            PlayerAuthResponse::read(data)?,
        )),
        KickPlayer::PACKET_ID => Ok(LSPackets::KickPlayer(KickPlayer::read(data)?)),
        RequestChars::PACKET_ID => Ok(LSPackets::RequestChars(RequestChars::read(data)?)),
        _ => {
            bail!("Unknown LS packet ID:0x{:02X}", data[0]);
        }
    }
}
