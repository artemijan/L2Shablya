use crate::common::dto::game_server::GSInfo;
use crate::common::dto::player::GSCharsInfo;
use crate::common::dto::{config, player};
use crate::common::message::Request;
use crate::crypt::rsa::{generate_rsa_key_pair, ScrambledRSAKeyPair};
use crate::packet::common::{
    PacketType, ReadablePacket, SendablePacket, ServerData, ServerStatus, ServerType,
};
use crate::packet::error::PacketRun;
use crate::packet::login_fail::GSLogin;
use crate::packet::to_gs::RequestChars;
use crate::packet::GSLoginFailReasons;
use anyhow::bail;
use futures::future::join_all;
use rand::Rng;
use sqlx::__rt::timeout;
use std::collections::{hash_map::Entry, HashMap};
use std::io::Read;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::RwLock;

///
/// This is A hashmap where the key is GS id and the values is a Sender object to respond with oneshot.
type GsPipe = Arc<RwLock<HashMap<u8, Sender<(u8, Request)>>>>;

#[derive(Clone, Debug)]
pub struct Login {
    key_pairs: Vec<ScrambledRSAKeyPair>,
    config: Arc<config::Server>,
    game_servers: Arc<RwLock<HashMap<u8, GSInfo>>>,
    players: Arc<RwLock<HashMap<String, player::Info>>>,
    gs_channels: GsPipe,
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
    pub async fn get_server_list(&self, client_ip: Ipv4Addr) -> Vec<ServerData> {
        let mut servers = Vec::new();
        let collection_lock = self.game_servers.read().await;
        for s in collection_lock.values() {
            servers.push(ServerData {
                ip: s.get_host_ip(client_ip),
                port: i32::from(s.get_port()),
                age_limit: i32::from(s.get_age_limit()),
                pvp: s.is_pvp(),
                current_players: 0,               //todo: implement me
                max_players: s.get_max_players(), //allow wrapping
                brackets: s.show_brackets(),
                clock: false, //todo: implement me
                status: ServerStatus::try_from(s.get_status()).ok(),
                server_id: i32::from(s.get_id()),
                server_type: s.get_server_type(),
            });
        }
        servers
    }
    pub async fn update_gs_status(&self, gs_id: u8, gs_info: GSInfo) -> Result<(), anyhow::Error> {
        let mut servers = self.game_servers.write().await;
        if servers.contains_key(&gs_id) {
            servers.insert(gs_info.get_id(), gs_info);
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
        if let Entry::Vacant(e) = servers.entry(gs_info.get_id()) {
            servers.insert(gs_info.get_id(), gs_info);
            Ok(())
        } else {
            Err(PacketRun {
                msg: Some(format!(
                    "GS already registered with id: {:}",
                    gs_info.get_id()
                )),
                response: Some(Box::new(GSLogin::new(
                    GSLoginFailReasons::AlreadyRegistered,
                ))),
            })
        }
    }

    pub async fn on_player_login(
        &self,
        mut player_info: player::Info,
    ) -> anyhow::Result<(), PacketRun> {
        let account_name = player_info.account_name.clone();
        let packet = Box::new(RequestChars::new(&account_name));
        let mut tasks = vec![];
        let timeout_duration = Duration::from_secs(u64::from(
            self.config.listeners.game_servers.messages.timeout,
        ));
        for gsi in self.game_servers.read().await.values() {
            let task = self.send_message_to_gs(gsi.get_id(), &account_name, packet.clone());
            tasks.push(timeout(timeout_duration, task));
        }
        let mut task_results = join_all(tasks).await.into_iter();
        while let Some(Ok(resp)) = task_results.next() {
            if let Ok(Some((gs_id, PacketType::ReplyChars(p)))) = resp {
                player_info.chars_on_servers.insert(
                    gs_id,
                    GSCharsInfo {
                        char_list: p.char_list,
                        chars_to_delete: p.chars_to_delete,
                        chars: p.chars,
                    },
                );
            }
            // ignore all of the tasks that are timed out
        }
        let mut players_lock = self.players.write().await;
        players_lock.insert(account_name, player_info); // if player exists it will drop
        Ok(())
    }
    pub async fn get_player(&self, account_name: &str) -> Option<player::Info> {
        self.players.read().await.get(account_name).cloned()
    }
    pub async fn send_message_to_gs(
        &self,
        gs_id: u8,
        message_id: &str,
        packet: Box<dyn SendablePacket>,
    ) -> anyhow::Result<Option<(u8, PacketType)>> {
        let sender: Sender<(u8, Request)>;
        let (resp_tx, resp_rx) = oneshot::channel();
        {
            let channels = self.gs_channels.read().await;
            sender = channels.get(&gs_id).unwrap().clone(); //we can make as many sender copies as we want
        }
        let message = Request {
            response: resp_tx,
            body: packet,
            sent_at: SystemTime::now(),
            id: message_id.to_string(),
        };
        sender.send((gs_id, message)).await?;
        let k = resp_rx
            .await
            .expect("Can not send the message to game server");
        Ok(k)
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
    pub async fn connect_gs(&self, server_id: u8, gs_channel: Sender<(u8, Request)>) {
        self.gs_channels
            .write()
            .await
            .entry(server_id)
            .or_insert(gs_channel);
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
