use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub server: Server,
}

impl ServerConfig {
    pub fn load(file_name: &str) -> Self {
        let file = File::open(file_name)
            .unwrap_or_else(|e| panic!("Failed to open config file: {}. Error: {}", file_name, e));
        let reader = BufReader::new(file);
        let config: ServerConfig = serde_yaml::from_reader(reader).unwrap_or_else(|e| {
            panic!(
                "Unable to parse {}, the format is incorrect, {}",
                file_name, e
            )
        });
        println!(
            "Configuration ok, starting application: {}",
            config.server.name
        );
        config
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    #[serde(rename = "url")]
    pub url: String,
    pub max_connections: u8,
    pub min_connections: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub name: String,
    pub blowfish_key: String,
    pub listeners: Listeners,
    pub database: Database,
    pub client: Client,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Listener {
    pub ip: String,
    pub port: u16,
    pub packet_read_timeout: usize,
    pub reuse_addr: bool,
    pub reuse_port: bool,
    pub no_delay: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Listeners {
    pub game_servers: Listener,
    pub clients: Listener,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Client {
    pub timeout: u8,
    pub enable_cmdline_login: bool,
}
