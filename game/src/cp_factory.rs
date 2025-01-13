use crate::client_thread::ClientHandler;
use crate::packets::from_client::auth::AuthLogin;
use crate::packets::from_client::new_char::NewCharacter;
use crate::packets::from_client::noop::NoOp;
use crate::packets::from_client::protocol::ProtocolVersion;
use crate::packets::HandleablePacket;
use l2_core::shared_packets::common::ReadablePacket;
use tracing::error;

pub fn build_client_packet(
    data: &[u8],
) -> Option<Box<dyn HandleablePacket<HandlerType = ClientHandler>>> {
    if data.is_empty() {
        return None;
    }
    match data[0] {
        ProtocolVersion::PACKET_ID => Some(Box::new(ProtocolVersion::read(data)?)),
        AuthLogin::PACKET_ID => Some(Box::new(AuthLogin::read(data)?)),
        NewCharacter::PACKET_ID => Some(Box::new(NewCharacter::read(data)?)),
        0xD0 => build_ex_client_packet(&data[1..data.len()]),
        _ => {
            error!("Unknown GS packet ID:0x{:02X}", data[0]);
            Some(Box::new(NoOp::read(data)?))
        }
    }
}

pub fn build_ex_client_packet(
    data: &[u8],
) -> Option<Box<dyn HandleablePacket<HandlerType = ClientHandler>>> {
    if data.len() < 2 {
        return None;
    }
    let packet_id = u16::from_le_bytes([data[0], data[1]]);
    match packet_id {
        0x01 => Some(Box::new(ProtocolVersion::read(data)?)),
        0x02 => Some(Box::new(ProtocolVersion::read(data)?)),
        _ => {
            error!("Unknown GS packet ID:0x{:02X}", data[0]);
            Some(Box::new(NoOp::read(data)?))
        }
    }
}
