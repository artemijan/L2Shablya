use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::BufReader;
use crate::common::dto::{Connection, Database, Runtime};

#[derive(Debug, Clone, Deserialize)]
pub struct GSServer {
    pub name: String,
    pub blowfish_key: String,
    pub runtime: Option<Runtime>,
    pub listeners: Listeners,
    pub database: Database,
    pub client: Client,
}

impl GSServer {
    pub fn load(file_name: &str) -> Self {
        let file = File::open(file_name)
            .unwrap_or_else(|e| panic!("Failed to open config file: {file_name}. Error: {e}"));
        let reader = BufReader::new(file);
        let config: GSServer = serde_yaml::from_reader(reader).unwrap_or_else(|e| {
            panic!("Unable to parse {file_name}, the format is incorrect, {e}")
        });
        println!("Configuration ok, starting application: {}", config.name);
        config
    }
    pub fn from_string(conf: &str) -> Self {
        serde_yaml::from_str::<GSServer>(conf)
            .unwrap_or_else(|e| panic!("Unable to parse {conf}, the format is incorrect, {e}"))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientListener {
    pub connection: Connection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Listeners {
    pub clients: ClientListener,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Client {
    pub timeout: u8,
}
