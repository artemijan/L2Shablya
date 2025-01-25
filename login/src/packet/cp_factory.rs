use anyhow::bail;
use crate::client_thread::ClientHandler;
use crate::packet::from_client::{
    RequestAuthGG, RequestAuthLogin, RequestGSLogin, RequestServerList,
};
use crate::packet::HandleablePacket;
use l2_core::crypt::rsa::ScrambledRSAKeyPair;
use l2_core::shared_packets::common::ReadablePacket;

/// Client Packet Factory
pub fn build_client_packet(
    data: &[u8],
    key_pair: &ScrambledRSAKeyPair,
) -> anyhow::Result<Box<dyn HandleablePacket<HandlerType = ClientHandler>>> {
    if data.is_empty() {
        bail!("Empty packet");
    }
    let (opcode, packet_body) = data.split_at(1);
    match opcode[0] {
        RequestAuthLogin::PACKET_ID => {
            let (raw1, rest) = packet_body.split_at(128);
            let decr_raw1 = key_pair.decrypt_data(raw1)?;
            let mut decrypted = decr_raw1;
            let mut is_new_auth = false;
            if packet_body.len() >= 256 {
                let (raw2, _) = rest.split_at(128);
                let decr_raw2 = key_pair.decrypt_data(raw2)?;
                decrypted = [decrypted, decr_raw2].concat();
                is_new_auth = true;
            }
            decrypted.push(u8::from(is_new_auth));
            Ok(Box::new(RequestAuthLogin::read(&decrypted)?))
        }
        RequestAuthGG::PACKET_ID => Ok(Box::new(RequestAuthGG::read(packet_body)?)),
        // 0x0B => Some(), //cmd login
        RequestGSLogin::PACKET_ID => Ok(Box::new(RequestGSLogin::read(packet_body)?)),
        RequestServerList::PACKET_ID => {
            Ok(Box::new(RequestServerList::read(packet_body)?))
        }
        // 0x0E => Some(LoginClientOpcodes::RequestPiAgreementCheck),
        // 0x0F => Some(LoginClientOpcodes::RequestPiAgreement),
        _ => {
            bail!("Unknown Client packet ID:0x{:02X}", data[0]);
        }
    }
}
