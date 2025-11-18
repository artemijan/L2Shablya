use crate::ls_client::LoginServerClient;
use bytes::{Bytes, BytesMut};
use kameo::message::{Context, Message};
use tracing::instrument;
use l2_core::crypt::generate_blowfish_key;
use l2_core::crypt::login::Encryption;
use l2_core::crypt::rsa::RSAPublicKey;
use l2_core::shared_packets::gs_2_ls::BlowFish;
use l2_core::shared_packets::{gs_2_ls::RequestAuthGS, ls_2_gs::InitLS};
use l2_core::traits::ServerToServer;

impl Message<InitLS> for LoginServerClient {
    type Reply = anyhow::Result<()>;
    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: InitLS,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let config = self.controller.get_cfg();
        let ra = RequestAuthGS::new(&config)?; //todo: put it in the cache
        let p_key = RSAPublicKey::from_modulus(&msg.public_key)?;
        let new_key = generate_blowfish_key(Some(40));
        let encrypted_data = BytesMut::from(Bytes::from(p_key.encrypt(&new_key)));
        let bf = BlowFish::new(encrypted_data);
        self.send_packet(bf).await?;
        self.set_blowfish(Encryption::new(&new_key));
        self.send_packet(ra).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::GameController;
    use crate::test_utils::test::spawn_ls_client_actor;
    use l2_core::config::gs::GSServerConfig;
    use l2_core::crypt::rsa::ScrambledRSAKeyPair;
    use l2_core::traits::ServerConfig;
    use ntest::timeout;
    use std::sync::Arc;
    use test_utils::utils::get_test_db;
    use tokio::io::{split, AsyncReadExt};

    #[tokio::test]
    async fn test_handle() {
        let pool = get_test_db().await;
        let the_key = ScrambledRSAKeyPair::from_pem(include_str!(
            "../../../../test_data/test_private_key.pem"
        ))
        .unwrap();
        let pack = InitLS::new(the_key.get_modulus());
        let (mut client, server) = tokio::io::duplex(1024);
        let (r, w) = split(server);
        let cfg = Arc::new(GSServerConfig::from_string(include_str!(
            "../../../../config/game.yaml"
        )));
        let controller = Arc::new(GameController::from_config(cfg).await);
        controller.add_online_account("test", None);
        let ls_actor = spawn_ls_client_actor(controller, pool, r, w).await;
        let res = ls_actor.ask(pack).await;
        assert!(res.is_ok());
        let mut resp = [0; 146];
        client.read_exact(&mut resp).await.unwrap();
        assert_eq!([146, 0], resp[..2]);
        let mut resp2 = [0; 42];
        client.read_exact(&mut resp2).await.unwrap();
        assert_eq!([42, 0], resp2[..2]);
    }
}
