use crate::dto::game_server::GSInfo;
use crate::dto::player;
use dashmap::DashMap;
use l2_core::config::login;
use l2_core::crypt::rsa::{generate_rsa_key_pair, ScrambledRSAKeyPair};
use l2_core::message_broker::MessageBroker;
use l2_core::shared_packets::common::PacketType;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

#[derive(Debug)]
pub struct LoginController {
    key_pairs: Vec<ScrambledRSAKeyPair>,
    pub(super) config: Arc<login::LoginServer>,
    pub(super) game_servers: DashMap<u8, GSInfo>,
    pub(super) ip_ban_list: DashMap<String, i64>,
    pub(super) players: DashMap<String, player::Info>,
    pub message_broker: Arc<MessageBroker<u8, PacketType>>,
}

impl LoginController {
    pub fn new(config: Arc<login::LoginServer>) -> Self {
        info!("Loading LoginController...");
        let threshold =
            Duration::from_secs(u64::from(config.listeners.game_servers.messages.timeout));
        Self {
            key_pairs: Self::generate_rsa_key_pairs(10),
            config,
            ip_ban_list: DashMap::new(),
            players: DashMap::new(),
            game_servers: DashMap::new(),
            message_broker: MessageBroker::new(threshold),
        }
    }
    pub fn get_config(&self) -> &login::LoginServer {
        &self.config
    }
    pub fn get_game_server(&self, gs_id: u8) -> Option<GSInfo> {
        self.game_servers.get(&gs_id).map(|gs| gs.clone())
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
    use l2_core::{config::login::LoginServer, traits::ServerConfig};
    use ntest::timeout;

    #[tokio::test]
    #[timeout(2000)]
    async fn test_login_controller() {
        let config = Arc::new(LoginServer::load("../test_data/test_config.yaml"));
        let controller = LoginController::new(config);
        let gs = controller.get_game_server(1);
        assert!(gs.is_none());
        assert!(controller.players.is_empty());
        assert!(controller.ip_ban_list.is_empty());
        assert!(controller.message_broker.packet_handlers.is_empty());
        assert_eq!(controller.get_random_rsa_key_pair().get_modulus().len(), 129);
    }
}