use crate::gs_thread::{enums, GSHandler};
use crate::packet::HandleablePacket;
use async_trait::async_trait;
use l2_core::shared_packets::gs_2_ls::BlowFish;

#[async_trait]
impl HandleablePacket for BlowFish {
    type HandlerType = GSHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let mut key = self.encrypted_key.clone();
        let mut decrypted = gs.decrypt_rsa(&mut key)?;
        // there are nulls before the key we must remove them
        if let Some(index) = decrypted.iter().position(|&x| x != 0) {
            decrypted.drain(..index);
        }
        gs.set_connection_state(&enums::GS::BfConnected).await?;
        gs.set_blowfish_key(&decrypted)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::LoginController;
    use crate::gs_thread::enums::GS;
    use crate::gs_thread::GSHandler;
    use crate::packet::HandleablePacket;
    use l2_core::config::login::LoginServer;
    use l2_core::crypt::rsa::ScrambledRSAKeyPair;
    use l2_core::shared_packets::gs_2_ls::BlowFish;
    use l2_core::traits::handlers::PacketHandler;
    use l2_core::traits::ServerConfig;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::split;
    const KEY: [u8; 128] = [
        64, 204, 20, 111, 162, 78, 78, 210, 17, 231, 112, 227, 226, 84, 147, 93, 64, 79, 193, 26,
        89, 82, 228, 32, 22, 49, 255, 229, 142, 165, 176, 38, 231, 28, 154, 28, 74, 149, 147, 36,
        197, 17, 61, 100, 195, 255, 250, 161, 55, 133, 83, 219, 127, 25, 230, 168, 38, 2, 106, 37,
        143, 203, 219, 16, 33, 154, 27, 74, 201, 226, 215, 252, 22, 134, 218, 74, 254, 10, 45, 91,
        90, 103, 40, 190, 83, 147, 75, 149, 247, 195, 157, 82, 161, 29, 2, 220, 119, 24, 20, 33,
        187, 175, 119, 85, 195, 174, 165, 248, 203, 59, 182, 68, 133, 151, 103, 216, 254, 195, 153,
        250, 218, 93, 200, 146, 248, 14, 228, 105,
    ];
    #[tokio::test]
    async fn handler_gs_ok() {
        let p_key = ScrambledRSAKeyPair::from_pem(include_str!(
            "../../../../test_data/test_private_key.pem"
        ))
        .unwrap();
        let pub_key = p_key.to_public_key();
        let scr = ScrambledRSAKeyPair::new((p_key, pub_key));

        let packet = BlowFish::new(KEY.to_vec());
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool, cloned_lc);
        ch.set_rsa_key(scr);
        ch.set_connection_state(&GS::Connected).await.unwrap();
        let res = packet.handle(&mut ch).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn handler_gs_invalid_blowfish() {
        let packet = BlowFish::new(KEY.to_vec());
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool, cloned_lc);
        ch.set_connection_state(&GS::Connected).await.unwrap();
        let res = packet.handle(&mut ch).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn handler_gs_connection_state() {
        let p_key = ScrambledRSAKeyPair::from_pem(include_str!(
            "../../../../test_data/test_private_key.pem"
        ))
        .unwrap();
        let pub_key = p_key.to_public_key();
        let scr = ScrambledRSAKeyPair::new((p_key, pub_key));
        let packet = BlowFish::new(KEY.to_vec());
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServer::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let cloned_lc = lc.clone();
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut ch = GSHandler::new(r, w, ip, db_pool, cloned_lc);
        ch.set_rsa_key(scr);
        let res = packet.handle(&mut ch).await;
        assert!(res.is_err());
    }
}
