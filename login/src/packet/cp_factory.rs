use crate::packet::from_client::{
    RequestAuthGG, RequestAuthLogin, RequestGSLogin, RequestServerList,
};
use anyhow::bail;
use bytes::{BufMut, BytesMut};
use l2_core::crypt::rsa::ScrambledRSAKeyPair;
use l2_core::shared_packets::common::ReadablePacket;
use macro_common::PacketEnum;
use strum::Display;

#[derive(Clone, Debug, Display, PacketEnum)]
pub enum ClientPackets {
    RequestAuthLogin(RequestAuthLogin),
    RequestAuthGG(RequestAuthGG),
    RequestGSLogin(RequestGSLogin),
    RequestServerList(RequestServerList),
}

pub fn build_client_message_packet(
    mut data: BytesMut,
    key_pair: &ScrambledRSAKeyPair,
) -> anyhow::Result<ClientPackets> {
    if data.is_empty() {
        bail!("Empty packet");
    }
    let opcode = data.split_to(1);
    match opcode[0] {
        RequestAuthLogin::PACKET_ID => {
            let raw1 = data.split_to(128);
            let mut decrypted = key_pair.decrypt_data(&raw1)?;
            let mut is_new_auth = false;
            if data.len() >= 256 {
                let raw2 = data.split_to(128);
                let decr_raw2 = key_pair.decrypt_data(&raw2)?;
                decrypted.extend_from_slice(&decr_raw2);
                is_new_auth = true;
            }
            decrypted.put_u8(u8::from(is_new_auth));
            Ok(ClientPackets::RequestAuthLogin(RequestAuthLogin::read(
                decrypted,
            )?))
        }
        RequestAuthGG::PACKET_ID => Ok(ClientPackets::RequestAuthGG(RequestAuthGG::read(data)?)),
        RequestGSLogin::PACKET_ID => Ok(ClientPackets::RequestGSLogin(RequestGSLogin::read(data)?)),
        RequestServerList::PACKET_ID => Ok(ClientPackets::RequestServerList(
            RequestServerList::read(data)?,
        )),
        _ => {
            bail!("Unknown Client packet ID:0x{:02X}", data[0]);
        }
    }
}
