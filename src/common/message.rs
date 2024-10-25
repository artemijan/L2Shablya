use std::time::SystemTime;
use crate::packet::common::{PacketType, ReadablePacket, SendablePacket};
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub struct Request {
    pub response: Sender<Option<PacketType>>,
    pub body: Box<dyn SendablePacket>,
    pub sent_at: SystemTime,
    pub id: String,
}
