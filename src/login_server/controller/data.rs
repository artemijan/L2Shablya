use std::collections::HashMap;
use std::sync::Arc;
use rand::Rng;
use tokio::sync::mpsc::Sender;
use tokio::sync::RwLock;
use crate::common::dto::{config, player};
use crate::common::dto::game_server::GSInfo;
use crate::common::message::Request;
use crate::crypt::rsa::{generate_rsa_key_pair, ScrambledRSAKeyPair};

type ConcurrentRequestMap = Arc<RwLock<HashMap<u8, Sender<(u8, Request)>>>>;
#[derive(Clone, Debug)]
pub struct Login {
    key_pairs: Vec<ScrambledRSAKeyPair>,
    pub(super) config: Arc<config::Server>,
    pub(super) game_servers: Arc<RwLock<HashMap<u8, GSInfo>>>,
    pub(super) players: Arc<RwLock<HashMap<String, player::Info>>>,
    pub(super) gs_channels: ConcurrentRequestMap,
}


impl Login {
    pub fn new(config: Arc<config::Server>) -> Login {
        println!("Loading LoginController...");
        Login {
            key_pairs: Login::generate_rsa_key_pairs(10),
            config,
            players: Arc::new(RwLock::new(HashMap::new())),
            game_servers: Arc::new(RwLock::new(HashMap::new())),
            gs_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn get_config(&self) -> &config::Server {
        &self.config
    }
    pub async fn get_game_server(&self, gs_id: u8) -> Option<GSInfo> {
        self.game_servers.read().await.get(&gs_id).cloned()
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
        println!("Generated {count} RSA key pairs");
        key_pairs
    }
}
