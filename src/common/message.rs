use crate::packet::common::{PacketType, SendablePacket};
use std::time::SystemTime;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub struct Request {
    pub response: Option<Sender<Option<(u8, PacketType)>>>,
    pub body: Box<dyn SendablePacket>,
    pub sent_at: SystemTime,
    pub id: String,
}
