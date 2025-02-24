use crate::ls_thread::LoginHandler;
use crate::packets::HandleablePacket;
use async_trait::async_trait;
use l2_core::crypt::generate_blowfish_key;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::rsa::RSAPublicKey;
use l2_core::shared_packets::gs_2_ls::BlowFish;
use l2_core::shared_packets::{gs_2_ls::RequestAuthGS, ls_2_gs::InitLS};
use l2_core::traits::handlers::{PacketHandler, PacketSender};

#[async_trait]
impl HandleablePacket for InitLS {
    type HandlerType = LoginHandler;
    async fn handle(&self, gs: &mut Self::HandlerType) -> anyhow::Result<()> {
        let controller = gs.get_controller();
        let config = controller.get_cfg();
        let ra = RequestAuthGS::new(&config)?;
        let p_key = RSAPublicKey::from_modulus(&self.public_key)?;
        let new_key = generate_blowfish_key(Some(40));
        let encrypted_data = p_key.encrypt(&new_key);
        let bf = BlowFish::new(encrypted_data);
        gs.send_packet(Box::new(bf)).await?;
        gs.set_blowfish(Encryption::new(&new_key));
        gs.send_packet(Box::new(ra)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::Controller;
    use l2_core::config::gs::GSServer;
    use l2_core::crypt::rsa::ScrambledRSAKeyPair;
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use std::net::Ipv4Addr;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    #[timeout(3000)]
    async fn test_handle() {
        let pool = get_test_db().await;
        let the_key = ScrambledRSAKeyPair::from_pem(include_str!(
            "../../../../test_data/test_private_key.pem"
        ))
        .unwrap();
        let pack = InitLS::new(the_key.get_modulus());
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServer::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(Controller::new(cfg));
        controller.add_online_account(String::from("test"));
        let mut ch = LoginHandler::new(r, w, Ipv4Addr::LOCALHOST, pool, controller);
        pack.handle(&mut ch).await.unwrap();
        tokio::spawn(async move {
            ch.handle_client().await.unwrap();
        });
        let mut resp = [0; 146];
        client.read_exact(&mut resp).await.unwrap();
        assert_eq!([146, 0], resp[..2]);
        let mut resp2 = [0; 42];
        client.read_exact(&mut resp2).await.unwrap();
        assert_eq!(
            [
                42, 0
            ],
            resp2[..2]
        );
        client.shutdown().await.unwrap();
    }
}
