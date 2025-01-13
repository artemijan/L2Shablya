use crate::gs_thread::GSHandler;
use l2_core::shared_packets::{
    common::ReadablePacket,
    gs_2_ls::{
        BlowFish, ChangeAccessLevel, ChangePassword, GSStatusUpdate, PlayerAuthRequest,
        PlayerInGame, PlayerLogout, PlayerTracert, ReplyChars, RequestAuthGS, RequestTempBan,
    },
};
use tracing::error;
use crate::packet::HandleablePacket;

pub fn build_gs_packet(data: &[u8]) -> Option<Box<dyn HandleablePacket<HandlerType = GSHandler>>> {
    if data.is_empty() {
        error!("GSFactory: data too short");
        return None;
    }
    match data[0] {
        BlowFish::PACKET_ID => Some(Box::new(BlowFish::read(data)?)),
        RequestAuthGS::PACKET_ID => Some(Box::new(RequestAuthGS::read(data)?)),
        PlayerInGame::PACKET_ID => Some(Box::new(PlayerInGame::read(data)?)),
        PlayerLogout::PACKET_ID => Some(Box::new(PlayerLogout::read(data)?)),
        ChangeAccessLevel::PACKET_ID => Some(Box::new(ChangeAccessLevel::read(data)?)),
        PlayerAuthRequest::PACKET_ID => Some(Box::new(PlayerAuthRequest::read(data)?)),
        GSStatusUpdate::PACKET_ID => Some(Box::new(GSStatusUpdate::read(data)?)),
        PlayerTracert::PACKET_ID => Some(Box::new(PlayerTracert::read(data)?)),
        ReplyChars::PACKET_ID => Some(Box::new(ReplyChars::read(data)?)),
        RequestTempBan::PACKET_ID => Some(Box::new(RequestTempBan::read(data)?)),
        ChangePassword::PACKET_ID => Some(Box::new(ChangePassword::read(data)?)),
        // 0x0B => Some(), //cmd login
        // 0x02 => Some(LoginClientOpcodes::RequestServerLogin),
        // 0x0E => Some(LoginClientOpcodes::RequestPiAgreementCheck),
        // 0x0F => Some(LoginClientOpcodes::RequestPiAgreement),
        _ => {
            error!("Unknown GS packet ID:0x{:02X}", data[0]);
            None
        }
    }
}
