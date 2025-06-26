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
        bail!("Not enough data to build packet: {data:?}");
    }
    let opcode = data.split_to(1);
    match opcode[0] {
        RequestAuthLogin::PACKET_ID => {
            let (raw1, rest) = data.split_at(128);
            let decr_raw1 = key_pair.decrypt_data(raw1)?;
            let mut decrypted = decr_raw1;
            let mut is_new_auth = false;
            if data.len() >= 256 {
                let (raw2, _) = rest.split_at(128);
                let decr_raw2 = key_pair.decrypt_data(raw2)?;
                decrypted.put_slice(&decr_raw2);
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
            bail!("Unknown Client packet ID:0x{:02X}", opcode[0]);
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::packet::cp_factory::{build_client_message_packet, ClientPackets};
    use bytes::BytesMut;
    use l2_core::crypt::rsa::ScrambledRSAKeyPair;

    #[tokio::test]
    async fn test_build_client_message_packet() {
        let packet_bytes = &[
            0, 111, 125, 244, 145, 84, 105, 242, 208, 32, 190, 242, 250, 167, 184, 36, 251, 198,
            229, 162, 94, 164, 79, 87, 68, 170, 166, 176, 59, 40, 47, 27, 21, 25, 124, 150, 77, 89,
            181, 194, 116, 217, 110, 171, 209, 185, 77, 251, 96, 150, 93, 77, 252, 126, 12, 83,
            216, 199, 44, 212, 246, 101, 130, 122, 182, 243, 194, 146, 36, 40, 82, 243, 90, 25, 74,
            246, 47, 109, 37, 56, 212, 73, 43, 55, 160, 146, 76, 62, 32, 155, 81, 200, 83, 80, 74,
            192, 236, 142, 195, 1, 233, 42, 53, 176, 191, 251, 137, 116, 19, 216, 67, 43, 219, 71,
            199, 182, 215, 100, 56, 14, 72, 99, 39, 222, 240, 60, 93, 250, 227, 2, 137, 47, 122,
            247, 198, 200, 127, 195, 145, 4, 36, 217, 202, 40, 14, 60, 108, 223, 105, 93, 75, 251,
            208, 190, 162, 161, 229, 132, 42, 51, 87, 98, 80, 8, 186, 82, 88, 167, 103, 122, 13,
            195, 77, 123, 44, 220, 155, 160, 165, 190, 158, 33, 165, 66, 242, 21, 246, 171, 168,
            42, 84, 226, 106, 87, 18, 27, 148, 249, 170, 123, 122, 134, 21, 116, 104, 107, 61, 216,
            241, 249, 115, 160, 104, 100, 178, 171, 179, 221, 7, 232, 125, 192, 245, 13, 131, 39,
            207, 45, 123, 108, 196, 95, 55, 75, 104, 206, 89, 157, 39, 39, 156, 116, 100, 177, 248,
            92, 174, 21, 189, 35, 251, 208, 238, 82, 192, 125, 223, 53, 211, 170, 49, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 201, 60, 201, 172, 185, 36,
            197, 189, 152, 64, 89, 234, 166, 34, 61, 246, 0, 0, 0, 0, 97, 9, 131, 137, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let scrambl =
            ScrambledRSAKeyPair::from_pem(include_str!("../../../test_data/test_private_key.pem"))
                .unwrap();
        let p = build_client_message_packet(BytesMut::from(&packet_bytes[..]), &scrambl).unwrap();
        if let ClientPackets::RequestAuthLogin(p) = p {
            assert_eq!(&p.username, "admin");
            assert_eq!(&p.password, "admin");
        } else {
            panic!("Expected client packet to be RequestAuthLogin");
        }
    }
}
