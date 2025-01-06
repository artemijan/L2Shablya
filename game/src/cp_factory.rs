use crate::client_thread::ClientHandler;
use crate::packets::from_client::auth::AuthLogin;
use crate::packets::from_client::noop::NoOp;
use crate::packets::from_client::protocol::ProtocolVersion;
use crate::packets::HandleablePacket;
use l2_core::packets::common::ReadablePacket;
use tracing::error;

pub fn build_client_packet(
    data: &[u8],
) -> Option<Box<dyn HandleablePacket<HandlerType = ClientHandler>>> {
    if data.is_empty() {
        return None;
    }
    match data[0] {
        0x0E => Some(Box::new(ProtocolVersion::read(data)?)),
        0x2B => Some(Box::new(AuthLogin::read(data)?)), //auth login
        _ => {
            error!("Unknown GS packet ID:0x{:02X}", data[0]);
            Some(Box::new(NoOp::read(data)?))
        }
    }
}
