use anyhow::bail;
use bytes::BytesMut;
use l2_core::shared_packets::{
    common::ReadablePacket,
    gs_2_ls::{
        BlowFish, ChangeAccessLevel, ChangePassword, GSStatusUpdate, PlayerAuthRequest,
        PlayerInGame, PlayerLogout, PlayerTracert, ReplyChars, RequestAuthGS, RequestTempBan,
    },
};
use macro_common::PacketEnum;
use strum::Display;

#[derive(Clone, Debug, Display, PacketEnum)]
pub enum GSPackets {
    BlowFish(BlowFish),
    RequestAuthGS(RequestAuthGS),
    PlayerInGame(PlayerInGame),
    PlayerLogout(PlayerLogout),
    ChangeAccessLevel(ChangeAccessLevel),
    PlayerAuthRequest(PlayerAuthRequest),
    GSStatusUpdate(GSStatusUpdate),
    PlayerTracert(PlayerTracert),
    ReplyChars(ReplyChars),
    RequestTempBan(RequestTempBan),
    ChangePassword(ChangePassword),
}

pub fn build_gs_packet(data: BytesMut) -> anyhow::Result<GSPackets> {
    if data.len() < 2 {
        bail!("Not enough data to build packet: {data:?}");
    }
    match data[0] {
        BlowFish::PACKET_ID => Ok(GSPackets::BlowFish(BlowFish::read(data)?)),
        RequestAuthGS::PACKET_ID => Ok(GSPackets::RequestAuthGS(RequestAuthGS::read(data)?)),
        PlayerInGame::PACKET_ID => Ok(GSPackets::PlayerInGame(PlayerInGame::read(data)?)),
        PlayerLogout::PACKET_ID => Ok(GSPackets::PlayerLogout(PlayerLogout::read(data)?)),
        ChangeAccessLevel::PACKET_ID => {
            Ok(GSPackets::ChangeAccessLevel(ChangeAccessLevel::read(data)?))
        }
        PlayerAuthRequest::PACKET_ID => {
            Ok(GSPackets::PlayerAuthRequest(PlayerAuthRequest::read(data)?))
        }
        GSStatusUpdate::PACKET_ID => Ok(GSPackets::GSStatusUpdate(GSStatusUpdate::read(data)?)),
        PlayerTracert::PACKET_ID => Ok(GSPackets::PlayerTracert(PlayerTracert::read(data)?)),
        ReplyChars::PACKET_ID => Ok(GSPackets::ReplyChars(ReplyChars::read(data)?)),
        RequestTempBan::PACKET_ID => Ok(GSPackets::RequestTempBan(RequestTempBan::read(data)?)),
        ChangePassword::PACKET_ID => Ok(GSPackets::ChangePassword(ChangePassword::read(data)?)),
        _ => {
            bail!("Unknown GS packet ID:0x{:02X}", data[0]);
        }
    }
}
