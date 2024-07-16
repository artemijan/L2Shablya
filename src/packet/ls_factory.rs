use crate::crypt::rsa::ScrambledRSAKeyPair;
use crate::packet::from_client::{RequestAuthGG, RequestGSLogin, RequestAuthLogin, RequestServerList};
use crate::packet::common::ReadablePacket;
use crate::packet::common::ClientHandle;

pub fn build_client_packet(
    data: &[u8],
    key_pair: &ScrambledRSAKeyPair,
) -> Option<Box<dyn ClientHandle>> {
    if data.len() <= 1 {
        return None;
    }
    let (opcode, packet_body) = data.split_at(1);
    match opcode[0] {
        0x00 => {
            let (raw1, rest) = packet_body.split_at(128);
            let (raw2, _) = rest.split_at(128);
            let decr_raw1 = key_pair.decrypt_data(raw1).ok()?;
            let decr_raw2 = key_pair.decrypt_data(raw2).ok()?;
            let decrypted = [decr_raw1, decr_raw2].concat();
            Some(Box::new(RequestAuthLogin::read(&decrypted).unwrap()))
        }
        0x07 => Some(Box::new(RequestAuthGG::read(packet_body).unwrap())),
        // 0x0B => Some(), //cmd login
        0x02 => Some(Box::new(RequestGSLogin::read(packet_body).unwrap())),
        0x05 => Some(Box::new(RequestServerList::read(packet_body).unwrap())),
        // 0x0E => Some(LoginClientOpcodes::RequestPiAgreementCheck),
        // 0x0F => Some(LoginClientOpcodes::RequestPiAgreement),
        _ => None,
    }
}
