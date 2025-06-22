use crate::enums;
use crate::gs_client::GameServerClient;
use bytes::{Buf, BytesMut};
use kameo::message::{Context, Message};
use l2_core::shared_packets::gs_2_ls::BlowFish;
use tracing::instrument;

impl Message<BlowFish> for GameServerClient {
    type Reply = anyhow::Result<()>;

    #[instrument(skip(self, _ctx))]
    async fn handle(
        &mut self,
        msg: BlowFish,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> anyhow::Result<()> {
        let key = BytesMut::from(&msg.encrypted_key.clone()[..]);
        let mut decrypted = self.key_pair.decrypt_data(&key)?;
        // there are nulls before the key, we must remove them
        if let Some(index) = decrypted.iter().position(|&x| x != 0) {
            decrypted.advance(index);
        }
        self.set_connection_state(&enums::GS::BfConnected).await?;
        self.set_blowfish_key(&decrypted)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::controller::LoginController;
    use crate::enums::GS;
    use crate::gs_client::GameServerClient;
    use crate::test_utils::test::spawn_custom_gs_client_actor;
    use bytes::BytesMut;
    use l2_core::config::login::LoginServerConfig;
    use l2_core::crypt::rsa::ScrambledRSAKeyPair;
    use l2_core::shared_packets::gs_2_ls::BlowFish;
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
        let packet = BlowFish::new(BytesMut::from(&KEY[..]));
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut gs_client = GameServerClient::new(ip, lc.clone(), db_pool.clone());
        gs_client.key_pair = p_key;
        gs_client
            .set_connection_state(&GS::Connected)
            .await
            .unwrap();
        let gs_actor = spawn_custom_gs_client_actor(lc, db_pool, r, w, Some(gs_client)).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn handler_gs_invalid_blowfish() {
        let packet = BlowFish::new(BytesMut::from(&KEY[..]));
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut gs_client = GameServerClient::new(ip, lc.clone(), db_pool.clone());
        gs_client
            .set_connection_state(&GS::Connected)
            .await
            .unwrap();
        let gs_actor = spawn_custom_gs_client_actor(lc, db_pool, r, w, Some(gs_client)).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_err());
    }
    #[tokio::test]
    async fn handler_gs_connection_state() {
        let p_key = ScrambledRSAKeyPair::from_pem(include_str!(
            "../../../../test_data/test_private_key.pem"
        ))
        .unwrap();
        let packet = BlowFish::new(BytesMut::from(&KEY[..]));
        let db_pool = get_test_db().await;
        let (_client, server) = tokio::io::duplex(1024);
        let cfg = LoginServerConfig::from_string(include_str!("../../../../config/login.yaml"));
        let lc = Arc::new(LoginController::new(Arc::new(cfg)));
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let (r, w) = split(server);
        let mut gs_client = GameServerClient::new(ip, lc.clone(), db_pool.clone());

        gs_client.key_pair = p_key;
        let gs_actor = spawn_custom_gs_client_actor(lc, db_pool, r, w, Some(gs_client)).await;
        let res = gs_actor.ask(packet).await;
        assert!(res.is_err());
    }
}
