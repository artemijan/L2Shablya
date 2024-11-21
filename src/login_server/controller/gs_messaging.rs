use super::data::Login;
use crate::common::packets::common::{PacketType, SendablePacket};
use crate::login_server::message::Request;
use anyhow::Error;
use futures::future::join_all;
use std::time::{Duration, SystemTime};
use tokio::sync::oneshot;
use tokio::time::timeout;
use uuid::Uuid;

impl Login {
    pub async fn send_message_to_all_gs<F>(
        &self,
        msg_id: &str,
        packet_factory: F,
    ) -> Vec<anyhow::Result<Option<(u8, PacketType)>>>
    where
        F: Fn() -> Box<dyn SendablePacket>,
    {
        let mut tasks = vec![];
        let timeout_duration = Duration::from_secs(u64::from(
            self.config.listeners.game_servers.messages.timeout,
        ));
        for gs_id in &self.game_servers {
            let task = self.send_message_to_gs(gs_id.get_id(), msg_id, packet_factory());
            tasks.push(timeout(timeout_duration, task));
        }
        join_all(tasks)
            .await
            .into_iter()
            .map(|res| res.map_err(Error::from).and_then(|inner| inner))
            .collect()
    }
    pub async fn notify_all_gs<F>(&self, packet_factory: F) -> Vec<anyhow::Result<()>>
    where
        F: Fn() -> Box<dyn SendablePacket>,
    {
        let mut tasks = vec![];
        for gsi in &self.game_servers {
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
}
