use anyhow::bail;
use crate::gs_thread::GSHandler;
use l2_core::shared_packets::{
    common::ReadablePacket,
    gs_2_ls::{
        BlowFish, ChangeAccessLevel, ChangePassword, GSStatusUpdate, PlayerAuthRequest,
        PlayerInGame, PlayerLogout, PlayerTracert, ReplyChars, RequestAuthGS, RequestTempBan,
    },
};
use crate::packet::HandleablePacket;

pub fn build_gs_packet(data: &[u8]) -> anyhow::Result<Box<dyn HandleablePacket<HandlerType = GSHandler>>> {
    if data.is_empty() {
        bail!("GSFactory: data too short");
    }
    match data[0] {
        BlowFish::PACKET_ID => Ok(Box::new(BlowFish::read(data)?)),
        RequestAuthGS::PACKET_ID => Ok(Box::new(RequestAuthGS::read(data)?)),
        PlayerInGame::PACKET_ID => Ok(Box::new(PlayerInGame::read(data)?)),
        PlayerLogout::PACKET_ID => Ok(Box::new(PlayerLogout::read(data)?)),
        ChangeAccessLevel::PACKET_ID => Ok(Box::new(ChangeAccessLevel::read(data)?)),
        PlayerAuthRequest::PACKET_ID => Ok(Box::new(PlayerAuthRequest::read(data)?)),
        GSStatusUpdate::PACKET_ID => Ok(Box::new(GSStatusUpdate::read(data)?)),
        PlayerTracert::PACKET_ID => Ok(Box::new(PlayerTracert::read(data)?)),
        ReplyChars::PACKET_ID => Ok(Box::new(ReplyChars::read(data)?)),
        RequestTempBan::PACKET_ID => Ok(Box::new(RequestTempBan::read(data)?)),
        ChangePassword::PACKET_ID => Ok(Box::new(ChangePassword::read(data)?)),
        _ => {
            bail!("Unknown GS packet ID:0x{:02X}", data[0]);
        }
    }
}
