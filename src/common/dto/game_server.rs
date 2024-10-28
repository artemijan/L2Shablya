use std::net::Ipv4Addr;
use std::str::FromStr;
use anyhow::bail;
use ipnet::Ipv4Net;
use num::{BigInt};
use crate::packet::common::ServerType;

#[derive(Debug, Clone)]
pub struct ServerHost {
    pub ip: Ipv4Addr,
    pub subnet: Ipv4Net,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct GSInfo {
    id: u8,
    accept_alternative_id: bool,
    host_reserved: bool,
    port: u16,
    is_authed: bool,
    status: i32,
    is_pvp: bool,
    server_type: i32,
    age_limit: u8,
    show_brackets: bool,
    max_players: u32,
    hex_id: Vec<u8>,
    hosts: Vec<ServerHost>,
}

impl GSInfo {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u8,
        accept_alternative_id: bool,
        host_reserved: bool,
        port: u16,
        is_authed: bool,
        status: i32,
        is_pvp: bool,
        server_type: i32,
        age_limit: u8,
        show_brackets: bool,
        max_players: u32,
        hex_id: Vec<u8>,
        hosts: Vec<String>,
    ) -> anyhow::Result<Self> {
        let validated_hosts = Self::validate_server_hosts(hosts)?;
        Ok(GSInfo {
            id,
            accept_alternative_id,
            host_reserved,
            port,
            is_authed,
            status,
            is_pvp,
            server_type,
            age_limit,
            show_brackets,
            max_players,
            hex_id,
            hosts: validated_hosts,
        })
    }


    pub fn get_host_ip(&self, client_ip: Ipv4Addr) -> Ipv4Addr {
        for s in &self.hosts {
            if s.subnet.contains(&client_ip) {
                return s.ip;
            }
        }
        Ipv4Addr::new(127, 0, 0, 1)
    }
    pub fn hex(&self) -> String {
        BigInt::from_signed_bytes_be(&self.hex_id).to_str_radix(16)
    }

    pub fn get_id(&self) -> u8 {
        self.id
    }
    pub fn accept_alternative_id(&self) -> bool {
        self.accept_alternative_id
    }
    pub fn is_host_reserved(&self) -> bool {
        self.host_reserved
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
    pub fn is_authed(&self) -> bool {
        self.is_authed
    }
    pub fn get_status(&self) -> u8 {
        self.status as u8
    }
    pub fn is_pvp(&self) -> bool {
        self.is_pvp
    }
    pub fn get_server_type(&self) -> Option<ServerType> {
        ServerType::try_from(self.server_type).ok()
    }
    pub fn get_age_limit(&self) -> u8 {
        self.age_limit
    }
    pub fn show_brackets(&self) -> bool {
        self.show_brackets
    }
    pub fn get_max_players(&self) -> i32 {
        self.max_players as i32
    }
    pub fn set_max_players(&mut self, max_players: u32) {
        self.max_players = max_players;
    }
    pub fn set_age_limit(&mut self, age_limit: u8) {
        self.age_limit = age_limit;
    }
    pub fn use_square_brackets(&mut self, is_square_brackets: bool) {
        self.show_brackets = is_square_brackets;
    }
    pub fn set_server_type(&mut self, server_type: i32) {
        self.server_type = server_type;
    }
    pub fn set_server_status(&mut self, status: i32) {
        self.status = status;
    }
    pub fn get_hex_id(&self) -> Vec<u8> {
        self.hex_id.clone()
    }
    pub fn get_hosts(&self) -> Vec<ServerHost> {
        self.hosts.clone()
    }

    fn validate_server_hosts(hosts: Vec<String>) -> anyhow::Result<Vec<ServerHost>> {
        let mut validated_hosts = Vec::new();
        for host in hosts.chunks(2) {
            if host.len() != 2 {
                bail!("Incorrect host sent by game server {:?}.", host);
            }
            let subnet = Ipv4Net::from_str(&host[0])?;
            let ip = Ipv4Addr::from_str(&host[1])?;
            validated_hosts.push(ServerHost { ip, subnet });
        }
        // sort from narrow to wide
        validated_hosts.sort_by(
            |a, b| b.subnet.prefix_len().cmp(&a.subnet.prefix_len())
        );
        Ok(validated_hosts)
    }
}
