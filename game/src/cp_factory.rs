use crate::client_thread::ClientHandler;
use crate::packets::from_client::auth::AuthLogin;
use crate::packets::from_client::char_create::CreateCharRequest;
use crate::packets::from_client::extended::{CheckCharName, GoLobby};
use crate::packets::from_client::new_char_request::NewCharacterRequest;
use crate::packets::from_client::noop::NoOp;
use crate::packets::from_client::protocol::ProtocolVersion;
use crate::packets::HandleablePacket;
use anyhow::bail;
use l2_core::shared_packets::common::ReadablePacket;
use tracing::error;

pub fn build_client_packet(
    data: &[u8],
) -> anyhow::Result<Box<dyn HandleablePacket<HandlerType = ClientHandler>>> {
    if data.is_empty() {
        bail!("Empty packet");
    }
    let packet_body = &data[1..]; // skip 1st byte, because it's packet id
    match data[0] {
        ProtocolVersion::PACKET_ID => Ok(Box::new(ProtocolVersion::read(packet_body)?)),
        AuthLogin::PACKET_ID => Ok(Box::new(AuthLogin::read(packet_body)?)),
        NewCharacterRequest::PACKET_ID => Ok(Box::new(NewCharacterRequest::read(packet_body)?)),
        CreateCharRequest::PACKET_ID => Ok(Box::new(CreateCharRequest::read(packet_body)?)),
        0xD0 => build_ex_client_packet(packet_body),
        _ => {
            error!("Unknown GS packet ID:0x{:02X}", data[0]);
            Ok(Box::new(NoOp::read(data)?))
        }
    }
}

pub fn build_ex_client_packet(
    data: &[u8],
) -> anyhow::Result<Box<dyn HandleablePacket<HandlerType = ClientHandler>>> {
    if data.len() < 2 {
        bail!("Empty extended packet {data:?}");
    }
    let packet_id = u16::from_le_bytes([data[0], data[1]]);
    let packet_body = &data[2..];
    match Some(packet_id) {
        GoLobby::EX_PACKET_ID => Ok(Box::new(GoLobby::read(packet_body)?)),
        CheckCharName::EX_PACKET_ID => Ok(Box::new(CheckCharName::read(packet_body)?)),
        _ => {
            error!("Unknown extended GS packet ID:0x{:02X}", data[0]);
            Ok(Box::new(NoOp::read(data)?))
        }
    }
}
