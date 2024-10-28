use crate::packet::common::GSHandle;
use crate::packet::common::ReadablePacket;
use crate::packet::from_gs::{BlowFish, PlayerTracert};
use crate::packet::from_gs::{GSStatusUpdate, PlayerAuthRequest, PlayerInGame, PlayerLogout, ReplyChars, GS};

pub fn build_gs_packet(data: &[u8]) -> Option<Box<dyn GSHandle>> {
    if data.len() <= 1 {
        return None;
    }
    match data[0] {
        0x00 => Some(Box::new(BlowFish::read(data)?)),
        0x01 => Some(Box::new(GS::read(data)?)),
        0x03 => Some(Box::new(PlayerLogout::read(data)?)),
        0x02 => Some(Box::new(PlayerInGame::read(data)?)),
        0x06 => Some(Box::new(GSStatusUpdate::read(data)?)),
        0x08 => Some(Box::new(ReplyChars::read(data)?)),
        0x05 => Some(Box::new(PlayerAuthRequest::read(data)?)),
        0x07 => Some(Box::new(PlayerTracert::read(data)?)),
        // 0x0B => Some(), //cmd login
        // 0x02 => Some(LoginClientOpcodes::RequestServerLogin),
        // 0x0E => Some(LoginClientOpcodes::RequestPiAgreementCheck),
        // 0x0F => Some(LoginClientOpcodes::RequestPiAgreement),
        _ => None,
    }
}
