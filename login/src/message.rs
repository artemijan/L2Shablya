use std::time::SystemTime;
use tokio::sync::oneshot::Sender;

use l2_core::packets::common::{PacketType, SendablePacket};

#[derive(Debug)]
pub struct Request {
    pub response: Option<Sender<Option<(u8, PacketType)>>>,
    pub body: Option<Box<dyn SendablePacket>>,
    pub sent_at: SystemTime,
    pub id: String,
}
