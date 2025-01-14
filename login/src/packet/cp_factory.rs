use crate::client_thread::ClientHandler;
use crate::packet::from_client::{
    RequestAuthGG, RequestAuthLogin, RequestGSLogin, RequestServerList,
};
use crate::packet::HandleablePacket;
use l2_core::crypt::rsa::ScrambledRSAKeyPair;
use l2_core::shared_packets::common::ReadablePacket;
use tracing::error;

/// Client Packet Factory
pub fn build_client_packet(
    data: &[u8],
    key_pair: &ScrambledRSAKeyPair,
) -> Option<Box<dyn HandleablePacket<HandlerType = ClientHandler>>> {
    if data.is_empty() {
        return None;
    }
    let (opcode, packet_body) = data.split_at(1);
    match opcode[0] {
        RequestAuthLogin::PACKET_ID => {
            let (raw1, rest) = packet_body.split_at(128);
            let decr_raw1 = key_pair.decrypt_data(raw1).ok()?;
            let mut decrypted = decr_raw1;
            let mut is_new_auth = false;
            if packet_body.len() >= 256 {
                let (raw2, _) = rest.split_at(128);
                let decr_raw2 = key_pair.decrypt_data(raw2).ok()?;
                decrypted = [decrypted, decr_raw2].concat();
                is_new_auth = true;
            }
            decrypted.push(u8::from(is_new_auth));
            Some(Box::new(RequestAuthLogin::read(&decrypted).unwrap()))
        }
        RequestAuthGG::PACKET_ID => Some(Box::new(RequestAuthGG::read(packet_body).unwrap())),
        // 0x0B => Some(), //cmd login
        RequestGSLogin::PACKET_ID => Some(Box::new(RequestGSLogin::read(packet_body).unwrap())),
        RequestServerList::PACKET_ID => {
            Some(Box::new(RequestServerList::read(packet_body).unwrap()))
        }
        // 0x0E => Some(LoginClientOpcodes::RequestPiAgreementCheck),
        // 0x0F => Some(LoginClientOpcodes::RequestPiAgreement),
        _ => {
            error!("Unknown Client packet ID:0x{:02X}", data[0]);
            None
        }
    }
}
