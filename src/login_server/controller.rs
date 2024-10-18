use crate::common::dto::config;
use crate::common::dto::game_server::GSInfo;
use crate::common::message::Message;
use crate::crypt::rsa::{generate_rsa_key_pair, ScrambledRSAKeyPair};
use crate::packet::common::{PacketType, ReadablePacket, SendablePacket};
use crate::packet::error::PacketRun;
use crate::packet::login_fail::{GSLogin, PlayerLogin};
use crate::packet::{GSLoginFailReasons, PlayerLoginFailReasons};
use anyhow::bail;
use rand::Rng;
use std::collections::{hash_map::Entry, HashMap};
use std::io::Read;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Clone, Debug)]
pub struct Login {
    key_pairs: Vec<ScrambledRSAKeyPair>,
    config: Arc<config::Server>,
    game_servers: Arc<RwLock<HashMap<u8, GSInfo>>>,
    gs_channels: Arc<RwLock<HashMap<u8, Sender<Message>>>>,
}

impl Login {
    pub fn new(config: Arc<config::Server>) -> Login {
        println!("Loading LoginController...");
        Login {
            key_pairs: Login::generate_rsa_key_pairs(10),
            config,
            game_servers: Arc::new(RwLock::new(HashMap::new())),
            gs_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn get_config(&self) -> &config::Server {
        &self.config
    }
    pub fn get_game_server(&self, gs_id: u8) -> Option<GSInfo> {
        self.game_servers.read().unwrap().get(&gs_id).cloned()
    }
    pub fn update_gs_status(&self, gs_id: u8, gs_info: GSInfo) -> Result<(), anyhow::Error> {
        let mut servers = self.game_servers.write().unwrap();
        if servers.contains_key(&gs_id) {
            servers.insert(gs_info.id, gs_info);
            Ok(())
        } else {
            bail!("Game server is not registered on login server.")
        }
    }

    pub fn register_gs(&self, gs_info: GSInfo) -> anyhow::Result<(), PacketRun> {
        let mut servers = self.game_servers.write().unwrap();
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

    pub fn remove_gs(&self, server_id: u8) {
        if let Ok(mut server_list) = self.game_servers.write() {
            server_list.remove(&server_id);
        }
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

    pub fn connect_gs(&self, server_id: u8, gs_channel: Sender<Message>) {
        self.gs_channels
            .write()
            .unwrap()
            .entry(server_id)
            .or_insert(gs_channel);
    }

    pub async fn send_message_to_gs(
        &self,
        gs_id: u8,
        packet: Box<dyn SendablePacket>,
        message_id: &str,
    ) -> anyhow::Result<Option<PacketType>> {
        let sender: Sender<Message>;
        let (resp_tx, resp_rx) = oneshot::channel();
        {
            let channels = self.gs_channels.read().unwrap();
            sender = channels.get(&gs_id).unwrap().clone(); //we can make as many sender copies as we want
        }
        let message = Message {
            response: resp_tx,
            request: packet,
            id: message_id.to_string(),
        };
        sender.send(message).await?;
        let k = resp_rx.blocking_recv()?;
        Ok(k)
    }
}
