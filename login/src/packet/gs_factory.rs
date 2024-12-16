use crate::gs_thread::GSHandler;
use l2_core::packets::{
    common::ReadablePacket,
    gs_2_ls::{
        BlowFish, ChangeAccessLevel, ChangePassword, GSStatusUpdate, PlayerAuthRequest,
        PlayerInGame, PlayerLogout, PlayerTracert, ReplyChars, RequestAuthGS, RequestTempBan,
    },
};
use tracing::error;
use crate::packet::HandleablePacket;

pub fn build_gs_packet(data: &[u8]) -> Option<Box<dyn HandleablePacket<HandlerType = GSHandler>>> {
    if data.len() <= 1 {
        error!("GSFactory: data too short");
        return None;
    }
    match data[0] {
        0x00 => Some(Box::new(BlowFish::read(data)?)),
        0x01 => Some(Box::new(RequestAuthGS::read(data)?)),
        0x02 => Some(Box::new(PlayerInGame::read(data)?)),
        0x03 => Some(Box::new(PlayerLogout::read(data)?)),
        0x04 => Some(Box::new(ChangeAccessLevel::read(data)?)),
        0x05 => Some(Box::new(PlayerAuthRequest::read(data)?)),
        0x06 => Some(Box::new(GSStatusUpdate::read(data)?)),
        0x07 => Some(Box::new(PlayerTracert::read(data)?)),
        0x08 => Some(Box::new(ReplyChars::read(data)?)),
        0x0A => Some(Box::new(RequestTempBan::read(data)?)),
        0x0B => Some(Box::new(ChangePassword::read(data)?)),
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
