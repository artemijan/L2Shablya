use crate::dto::game_server::GSInfo;
use crate::dto::player;
use crate::gs_client::GameServerClient;
use dashmap::DashMap;
use kameo::actor::ActorRef;
use l2_core::config::login;
use l2_core::crypt::rsa::{ScrambledRSAKeyPair, generate_rsa_key_pair};
use rand::Rng;
use std::sync::Arc;
use tracing::info;

#[derive(Debug)]
pub struct LoginController {
    key_pairs: Vec<ScrambledRSAKeyPair>,
    pub(super) config: Arc<login::LoginServerConfig>,
    pub game_servers: DashMap<u8, GSInfo>,
    pub(super) ip_ban_list: DashMap<String, i64>,
    pub players: DashMap<String, player::Info>,
    pub gs_actors: DashMap<u8, ActorRef<GameServerClient>>,
}

impl LoginController {
    pub fn new(config: Arc<login::LoginServerConfig>) -> Self {
        info!("Loading LoginController...");
        Self {
            key_pairs: Self::generate_rsa_key_pairs(10),
            config,
            ip_ban_list: DashMap::new(),
            players: DashMap::new(),
            game_servers: DashMap::new(),
            gs_actors: DashMap::new(),
        }
    }
    pub fn get_config(&self) -> &login::LoginServerConfig {
        &self.config
    }

    pub fn get_random_rsa_key_pair(&self) -> &ScrambledRSAKeyPair {
        let mut rng = rand::thread_rng();
        let random_number: usize = rng.gen_range(0..self.key_pairs.len());
        // safe to unwrap as we 100% sure that we load all keys when booting the application
        self.key_pairs
            .get(random_number)
            .expect("Can't access generated keys, seems like app is not properly booted")
    }

    fn generate_rsa_key_pairs(count: u8) -> Vec<ScrambledRSAKeyPair> {
        let mut key_pairs: Vec<ScrambledRSAKeyPair> = vec![];
        for _ in 0..count {
            let rsa_pair = generate_rsa_key_pair();
            let scrumbled = ScrambledRSAKeyPair::new(rsa_pair);
            key_pairs.push(scrumbled);
        }
        info!("Generated {count} RSA key pairs");
        key_pairs
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use l2_core::{config::login::LoginServerConfig, traits::ServerConfig};

    #[tokio::test]
    async fn test_login_controller() {
        let config = Arc::new(LoginServerConfig::load("../test_data/test_config.yaml"));
        let controller = LoginController::new(config);
        let gs = controller.game_servers.get(&1);
        assert!(gs.is_none());
        assert!(controller.players.is_empty());
        assert!(controller.ip_ban_list.is_empty());
        assert!(controller.gs_actors.is_empty());
        assert_eq!(
            controller.get_random_rsa_key_pair().get_modulus().len(),
            129
        );
    }
}
