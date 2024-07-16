use crate::common::message::Message;
use crate::crypt::rsa::{generate_rsa_key_pair, ScrambledRSAKeyPair};
use crate::packet::common::{PacketType, ReadablePacket, SendablePacket};
use anyhow::bail;
use rand::Rng;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use crate::common::dto::game_server::GameServerInfo;

#[derive(Clone, Debug)]
pub struct LoginController {
    key_pairs: Vec<ScrambledRSAKeyPair>,
    game_servers: Arc<RwLock<HashMap<u8, GameServerInfo>>>,
    gs_channels: Arc<RwLock<HashMap<u8, Sender<Message>>>>,
}

impl LoginController {
    pub fn new() -> LoginController {
        println!("Loading LoginController...");
        LoginController {
            key_pairs: LoginController::generate_rsa_key_pairs(10),
            game_servers: Arc::new(RwLock::new(HashMap::new())),
            gs_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn get_game_server(&self, gs_id: u8) -> Option<GameServerInfo> {
        self.game_servers.read().unwrap().get(&gs_id).cloned()
    }
    pub fn update_gs_status(
        &self,
        gs_id: u8,
        gs_info: GameServerInfo,
    ) -> Result<(), anyhow::Error> {
        let mut servers = self.game_servers.write().unwrap();
        if servers.contains_key(&gs_id) {
            servers.insert(gs_info.id, gs_info);
            Ok(())
        } else {
            bail!("Game server is not registered on login server.")
        }
    }

    #[allow(clippy::map_entry)]
    pub fn register_gs(&self, gs_info: GameServerInfo) -> Result<(), anyhow::Error> {
        let mut servers = self.game_servers.write().unwrap();
        if !servers.contains_key(&gs_info.id) {
            servers.insert(gs_info.id, gs_info);
            Ok(())
        } else {
            bail!("Game server already registered on login server.")
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
        println!("Generated {} RSA key pairs", count);
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
