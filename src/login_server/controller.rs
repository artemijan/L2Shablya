use crate::common::dto::config;
use crate::common::dto::game_server::GSInfo;
use crate::common::dto::player::Info;
use crate::crypt::rsa::{generate_rsa_key_pair, ScrambledRSAKeyPair};
use crate::packet::common::{ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use crate::packet::login_fail::{GSLogin, PlayerLogin};
use crate::packet::to_gs::RequestChars;
use crate::packet::{GSLoginFailReasons, PlayerLoginFailReasons};
use anyhow::bail;
use rand::Rng;
use std::collections::{hash_map::Entry, HashMap};
use std::io::Read;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::RwLock;
use tokio::task::JoinSet;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Login {
    key_pairs: Vec<ScrambledRSAKeyPair>,
    config: Arc<config::Server>,
    game_servers: Arc<RwLock<HashMap<u8, GSInfo>>>
}

impl Login {
    pub fn new(config: Arc<config::Server>) -> Login {
        println!("Loading LoginController...");
        Login {
            key_pairs: Login::generate_rsa_key_pairs(10),
            config,
            game_servers: Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub fn get_config(&self) -> &config::Server {
        &self.config
    }
    pub async fn get_game_server(&self, gs_id: u8) -> Option<GSInfo> {
        self.game_servers.read().await.get(&gs_id).cloned()
    }
    pub async fn update_gs_status(&self, gs_id: u8, gs_info: GSInfo) -> Result<(), anyhow::Error> {
        let mut servers = self.game_servers.write().await;
        if servers.contains_key(&gs_id) {
            servers.insert(gs_info.id, gs_info);
            Ok(())
        } else {
            bail!("Game server is not registered on login server.")
        }
    }

    pub async fn register_gs(&self, gs_info: GSInfo) -> anyhow::Result<(), PacketRun> {
        let mut servers = self.game_servers.write().await;
        if let Some(allowed_gs) = &self.config.allowed_gs {
            if !allowed_gs.contains_key(&gs_info.hex()) {
                return Err(PacketRun {
                    msg: Some(format!("GS wrong hex: {:}", gs_info.hex())),
                    response: Some(Box::new(GSLogin::new(GSLoginFailReasons::WrongHexId))),
                });
            }
        }
        if let Entry::Vacant(e) = servers.entry(gs_info.id) {
            servers.insert(gs_info.id, gs_info);
            Ok(())
        } else {
            Err(PacketRun {
                msg: Some(format!("GS already registered with id: {:}", gs_info.id)),
                response: Some(Box::new(GSLogin::new(
                    GSLoginFailReasons::AlreadyRegistered,
                ))),
            })
        }
    }

    pub async fn on_player_login(self: Arc<Self>, player_info: Info) {
        let servers = self.game_servers.read().await;
        let account_name = player_info.account_name.clone().unwrap();
        let packet = Box::new(RequestChars::new(&account_name));
    }

    pub async fn remove_gs(&self, server_id: u8) {
        let mut server_list = self.game_servers.write().await;
        server_list.remove(&server_id);
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
