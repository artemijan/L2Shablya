use crate::packet::common::{PacketType, ReadablePacket, SendablePacket};
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub struct Message {
    pub response: Sender<Option<PacketType>>,
    pub request: Box<dyn SendablePacket>,
    pub id: String,
}
