use crate::login_server::dto::game_server::GSInfo;
use crate::login_server::dto::{config, player};
use crate::login_server::message::Request;
use crate::crypt::rsa::{generate_rsa_key_pair, ScrambledRSAKeyPair};
use dashmap::DashMap;
use rand::Rng;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Login {
    key_pairs: Vec<ScrambledRSAKeyPair>,
    pub(super) config: Arc<config::Server>,
    pub(super) game_servers: DashMap<u8, GSInfo>,
    pub(super) ip_ban_list: DashMap<String, i64>,
    pub(super) players: DashMap<String, player::Info>,
    pub(super) gs_channels: DashMap<u8, Sender<(u8, Request)>>,
}

impl Login {
    pub fn new(config: Arc<config::Server>) -> Login {
        info!("Loading LoginController...");
        Login {
            key_pairs: Login::generate_rsa_key_pairs(10),
            config,
            ip_ban_list: DashMap::new(),
            players: DashMap::new(),
            game_servers: DashMap::new(),
            gs_channels: DashMap::new(),
        }
    }
    pub fn get_config(&self) -> &config::Server {
        &self.config
    }
    pub fn get_game_server(&self, gs_id: u8) -> Option<GSInfo> {
        self.game_servers.get(&gs_id).map(|gs| gs.clone())
    }
    pub fn get_random_rsa_key_pair(&self) -> ScrambledRSAKeyPair {
        let mut rng = rand::thread_rng();
        let random_number: usize = rng.gen_range(0..=9);
        self.key_pairs.get(random_number).unwrap().clone()
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
