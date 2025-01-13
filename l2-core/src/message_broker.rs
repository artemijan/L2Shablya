use crate::shared_packets::common::SendablePacket;
use crate::traits::handlers::PacketSender;
use anyhow::Error;
use dashmap::DashMap;
use futures::future::join_all;
use log::error;
use sea_orm::prelude::Uuid;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

#[derive(Debug)]
pub struct Request<K, V> {
    pub response: Option<oneshot::Sender<Option<(K, V)>>>,
    pub body: Option<Box<dyn SendablePacket>>,
    pub sent_at: SystemTime,
    pub id: String,
}

#[derive(Debug)]
pub struct MessageBroker<K, V> {
    pub inbox: DashMap<String, Request<K, V>>,
    pub timeout: Duration,
    pub sender: Sender<(u8, Request<K, V>)>,
    pub packet_handlers: DashMap<u8, Arc<dyn PacketSender>>,
}

impl<K, V> MessageBroker<K, V>
where
    K: Debug + Clone + Send + Sync + Eq + Hash + 'static,
    V: Debug + Clone + Send + Sync + 'static,
{
    #[must_use]
    pub fn new(timeout: Duration) -> Arc<Self> {
        let (rx, mut tx) = mpsc::channel::<(u8, Request<K, V>)>(100);
        let broker_inst = Arc::new(MessageBroker {
            inbox: DashMap::new(),
            timeout,
            sender: rx,
            packet_handlers: DashMap::new(),
        });
        let broker = broker_inst.clone();
        tokio::spawn(async move {
            loop {
                if let Some((sender_id, mut request)) = tx.recv().await {
                    //the message has been sent already, there is no sense to do it twice
                    let existing_msg = broker.inbox.remove(&request.id);
                    if let Some((_, existing_msg)) = existing_msg {
                        if let Some(resp) = existing_msg.response {
                            let _ = resp.send(None); // ignore error, we don't care if pipe is broken
                        }
                    }
                    //do a cleanup, if we have old messages, remove them
                    let now = SystemTime::now();

                    broker.inbox.retain(|_, req| {
                        now.duration_since(req.sent_at)
                            .is_ok_and(|elapsed| elapsed <= broker.timeout)
                    });
                    // send packet later, now we only remember it
                    let Some(req_body) = request.body.take() else {
                        error!("No body found for message {:?}", request);
                        continue;
                    };
                    let Some(packet_sender) = broker.packet_handlers.get(&sender_id) else {
                        error!("No packet handler found for sender_id {:?}", sender_id);
                        continue;
                    };
                    // we are safe to send bytes first and then update messages, there is a lock.
                    if packet_sender.send_packet(req_body).await.is_ok() {
                        broker.inbox.insert(request.id.clone(), request);
                    } else if let Some(resp) = request.response {
                        //if it wasn't successful then just send back NoResponse without storing it
                        let _ = resp.send(None);
                    }
                }
            }
        });
        broker_inst
    }

    pub fn respond_to_message(&self, receiver_id_option: Option<K>, message_id: &str, message: V) {
        let msg = self.inbox.remove(message_id);
        // TODO: once combining if let will be stable, refactor it
        if let Some((_msg_id, request)) = msg {
            if let Some(receiver_id) = receiver_id_option {
                if let Some(resp) = request.response {
                    let receiver_id_clone = receiver_id.clone();
                    if resp.send(Some((receiver_id, message))).is_err() {
                        error!("Failed to send response to message {message_id}, receiver is {receiver_id_clone:?}");
                    }
                }
            }
        }
        //if message is missing then we just ignore it
    }

    pub fn register_packet_handler(&self, receiver_id: u8, packet_sender: Arc<dyn PacketSender>) {
        self.packet_handlers.insert(receiver_id, packet_sender);
    }
    pub fn unregister_packet_handler(&self, sender_id: u8) {
        self.packet_handlers.remove(&sender_id);
    }

    pub async fn notify_all<F>(&self, packet_factory: F) -> Vec<anyhow::Result<()>>
    where
        F: Fn() -> Box<dyn SendablePacket>,
    {
        let mut tasks = vec![];
        for ph in &self.packet_handlers {
            let task = self.notify(*ph.key(), packet_factory());
            tasks.push(task);
        }
        join_all(tasks).await
    }

    ///
    /// # Errors
    /// - if the message was not sent
    pub async fn notify(
        &self,
        receiver_id: u8,
        packet: Box<dyn SendablePacket>,
    ) -> anyhow::Result<()> {
        let message = Request {
            response: None,
            body: Some(packet),
            sent_at: SystemTime::now(),
            id: Uuid::new_v4().to_string(),
        };
        self.sender.send((receiver_id, message)).await?;
        Ok(())
    }

    pub async fn send_message_to_all<F>(
        &self,
        msg_id: &str,
        packet_factory: F,
    ) -> Vec<anyhow::Result<Option<(K, V)>>>
    where
        F: Fn() -> Box<dyn SendablePacket>,
    {
        let mut tasks = vec![];
        for entry in &self.packet_handlers {
            let task = self.send_message(*entry.key(), msg_id, packet_factory());
            tasks.push(timeout(self.timeout, task));
        }
        join_all(tasks)
            .await
            .into_iter()
            .map(|res| res.map_err(Error::from).and_then(|inner| inner))
            .collect()
    }

    ///
    /// # Errors
    /// - if the message was not sent
    /// - if there was an error awaiting a response
    pub async fn send_message(
        &self,
        receiver_id: u8,
        message_id: &str,
        packet: Box<dyn SendablePacket>,
    ) -> anyhow::Result<Option<(K, V)>> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let message = Request {
            response: Some(resp_tx),
            body: Some(packet),
            sent_at: SystemTime::now(),
            id: message_id.to_string(),
        };
        self.sender.send((receiver_id, message)).await?;
        let k = resp_rx.await?;
        Ok(k)
    }
}
