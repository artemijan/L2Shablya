pub mod conversion;

use crate::crypt::login::Encryption;
use crate::dto::{Database, Runtime};
use crate::network::connection::{
    send_delayed_packet, send_packet, send_packet_blocking, ConnectionActor, HandleIncomingPacket,
};
use crate::network::encrypt_internal_packet;
use crate::shared_packets::write::SendablePacketBuffer;
use bytes::BytesMut;
use kameo::actor::ActorRef;
use kameo::message::Message;
use kameo::Actor;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

pub trait IpBan {
    fn is_ip_banned(&self, ip: &str) -> bool;
}

pub trait Shutdown {
    fn get_shutdown_listener(&self) -> Arc<Notify>;
    fn shutdown(&self) {
        self.get_shutdown_listener().notify_one();
    }
}

pub trait ServerConfig {
    fn load(path: &str) -> Self;
    fn from_string(content: &str) -> Self;
    fn runtime(&self) -> Option<&Runtime>;
    fn database(&self) -> &Database;
}

#[async_trait::async_trait]
pub trait ServerToServer: Actor + Message<HandleIncomingPacket> {
    fn get_packet_sender(&self) -> Option<&ActorRef<ConnectionActor<Self>>>;
    fn get_blowfish(&self) -> &Encryption;

    /// # Errors
    /// - fail to write padding
    ///
    fn get_bytes(&self, mut buffer: SendablePacketBuffer) -> anyhow::Result<BytesMut> {
        buffer.write_padding()?;
        let mut data = buffer.take();
        encrypt_internal_packet(&mut data, self.get_blowfish());
        Ok(data)
    }
    async fn send_packet(&self, buffer: SendablePacketBuffer) -> anyhow::Result<()> {
        let data = self.get_bytes(buffer)?;
        send_packet_blocking(self.get_packet_sender(), data.freeze()).await
    }
    async fn send_packet_no_wait(&self, buffer: SendablePacketBuffer) -> anyhow::Result<()> {
        let data = self.get_bytes(buffer)?;
        send_packet(self.get_packet_sender(), data.freeze()).await
    }
    async fn send_delayed_packet(
        &self,
        buffer: SendablePacketBuffer,
        delay: Duration,
    ) -> anyhow::Result<()> {
        let data = self.get_bytes(buffer)?;
        send_delayed_packet(self.get_packet_sender(), data.freeze(), delay).await
    }
}
