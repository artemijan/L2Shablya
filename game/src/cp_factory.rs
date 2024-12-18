use crate::client_thread::ClientHandler;
use crate::packets::HandleablePacket;
use tracing::error;

pub fn build_client_packet(
    data: &[u8],
) -> Option<Box<dyn HandleablePacket<HandlerType = ClientHandler>>> {
    if data.len() <= 1 {
        return None;
    }
    match data[0] {
        _ => {
            error!("Unknown GS packet ID:0x{:02X}", data[0]);
            None
        }
    }
}
