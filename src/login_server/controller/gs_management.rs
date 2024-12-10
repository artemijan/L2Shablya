use super::data::Login;
use crate::common::packets::common::{GSLoginFailReasons, ServerData, ServerStatus};
use crate::login_server::dto::game_server::GSInfo;
use crate::login_server::message::Request;
use std::net::Ipv4Addr;
use tokio::sync::mpsc::Sender;

impl Login {
    pub fn get_server_list(&self, client_ip: Ipv4Addr) -> Vec<ServerData> {
        let mut servers = Vec::new();
        for s in &self.game_servers {
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

    pub fn with_gs<F>(&self, gs_id: u8, f: F) -> bool
    where
        F: Fn(&mut GSInfo),
    {
        if let Some(mut gs) = self.game_servers.get_mut(&gs_id) {
            f(&mut gs);
            true
        } else {
            false
        }
    }
    fn generate_sequential_key(&self) -> anyhow::Result<u8, GSLoginFailReasons> {
        (0..=u8::MAX)
            .find(|key| !&self.game_servers.contains_key(key))
            .ok_or(GSLoginFailReasons::NoFreeID)
    }
    pub fn register_gs(&self, gs_info: GSInfo) -> anyhow::Result<u8, GSLoginFailReasons> {
        let id = gs_info.get_id();
        if let Some(allowed_gs) = &self.config.allowed_gs {
            if !allowed_gs.contains(&gs_info.hex_int()) {
                return Err(GSLoginFailReasons::WrongHexId);
            }
        }
        if self.game_servers.contains_key(&gs_info.get_id()) {
            if gs_info.accept_alternative_id() {
                return self.generate_sequential_key();
            }
            return Err(GSLoginFailReasons::AlreadyRegistered);
        }
        self.game_servers.insert(gs_info.get_id(), gs_info);
        Ok(id)
    }

    pub fn remove_gs(&self, server_id: u8) {
        self.game_servers.remove(&server_id);
    }

    pub fn connect_gs(&self, server_id: u8, gs_channel: Sender<(u8, Request)>) {
        self.gs_channels.entry(server_id).or_insert(gs_channel);
    }
}
