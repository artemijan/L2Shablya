use std::net::Ipv4Addr;
use std::time::{Duration, SystemTime};
use anyhow::{bail, Error};
use futures::future::join_all;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;
use crate::common::dto::game_server::GSInfo;
use crate::common::message::Request;
use crate::packet::common::{PacketType, SendablePacket, ServerData, ServerStatus};
use crate::packet::{error, GSLoginFailReasons};
use crate::packet::login_fail::GSLogin;
use super::data::Login;

impl Login {
    pub fn get_server_list(&self, client_ip: Ipv4Addr) -> Vec<ServerData> {
        let mut servers = Vec::new();
        for s in self.game_servers.iter() {
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

    pub fn with_gs<F>(&self, gs_id: &u8, f: F) -> bool
    where
        F: Fn(&mut GSInfo),
    {
        if let Some(mut gs) = self.game_servers.get_mut(gs_id) {
            f(&mut gs);
            true
        } else {
            false
        }
    }

    pub fn register_gs(&self, gs_info: GSInfo) -> anyhow::Result<(), error::PacketRun> {
        if let Some(allowed_gs) = &self.config.allowed_gs {
            if !allowed_gs.contains_key(&gs_info.hex()) {
                return Err(error::PacketRun {
                    msg: Some(format!("GS wrong hex: {:}", gs_info.hex())),
                    response: Some(Box::new(GSLogin::new(GSLoginFailReasons::WrongHexId))),
                });
            }
        }
        if !self.game_servers.contains_key(&gs_info.get_id()) {
            self.game_servers.insert(gs_info.get_id(), gs_info);
            Ok(())
        } else {
            Err(error::PacketRun {
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
    pub async fn send_message_to_all_gs<F>(&self, msg_id: &str, packet_factory: F) -> Vec<anyhow::Result<Option<(u8, PacketType)>>>
    where
        F: Fn() -> Box<dyn SendablePacket>,
    {
        let mut tasks = vec![];
        let timeout_duration = Duration::from_secs(u64::from(
            self.config.listeners.game_servers.messages.timeout,
        ));
        for gs_id in self.game_servers.iter() {
            let task = self.send_message_to_gs(gs_id.get_id(), msg_id, packet_factory());
            tasks.push(timeout(timeout_duration, task));
        }
        join_all(tasks).await.into_iter()
            .map(|res| {
                // Flatten each element by handling the outer Result
                res.map_err(Error::from) // Convert TimeoutError to anyhow::Error
                    .and_then(|inner| inner) // Flatten anyhow::Result<Option<(u8, PacketType)>> to anyhow::Result<Option<(u8, PacketType)>>
            })
            .collect()
    }
    pub async fn notify_all_gs<F>(&self, packet_factory: F) -> Vec<anyhow::Result<()>>
    where
        F: Fn() -> Box<dyn SendablePacket>,
    {
        let mut tasks = vec![];
        for gsi in self.game_servers.iter() {
            let task = self.notify_gs(gsi.get_id(), packet_factory());
            tasks.push(task);
        }
        join_all(tasks).await
    }
    pub async fn send_message_to_gs(
        &self,
        gs_id: u8,
        message_id: &str,
        packet: Box<dyn SendablePacket>,
    ) -> anyhow::Result<Option<(u8, PacketType)>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let sender_option = self.gs_channels.get(&gs_id);
        let sender = sender_option.unwrap();
        let message = Request {
            response: Some(resp_tx),
            body: packet,
            sent_at: SystemTime::now(),
            id: message_id.to_string(),
        };
        sender.send((gs_id, message)).await?;
        let k = resp_rx
            .await
            .expect("Can not receive an answer from game server");
        Ok(k)
    }
    pub async fn notify_gs(
        &self,
        gs_id: u8,
        packet: Box<dyn SendablePacket>,
    ) -> anyhow::Result<()> {
        let sender = self.gs_channels.get(&gs_id).unwrap();
        let message = Request {
            response: None,
            body: packet,
            sent_at: SystemTime::now(),
            id: Uuid::new_v4().to_string(),
        };
        sender.send((gs_id, message)).await?;
        Ok(())
    }

    pub async fn remove_gs(&self, server_id: u8) {
        self.game_servers.remove(&server_id);
    }

    pub fn connect_gs(&self, server_id: u8, gs_channel: Sender<(u8, Request)>) {
        self.gs_channels.entry(server_id).or_insert(gs_channel);
    }
}