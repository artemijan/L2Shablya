pub mod conversion;
use std::future::Future;
use crate::crypt::login::Encryption;
use crate::dto::{Database, Runtime};
use crate::network::connection::{
    send_delayed_packet, send_packet, send_packet_blocking, ConnectionActor, HandleIncomingPacket,
};
use crate::network::encrypt_internal_packet;
use crate::shared_packets::common::SendablePacket;
use bytes::Bytes;
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
fn get_bytes(encryption: &Encryption, packet: impl SendablePacket) -> anyhow::Result<Bytes> {
    let mut buffer = packet.get_buffer();
    buffer.write_padding()?;
    let mut data = buffer.take();
    encrypt_internal_packet(&mut data, encryption);
    Ok(data.freeze())
}

pub trait ServerToServer: Actor + Message<HandleIncomingPacket> {
    fn get_packet_sender(&self) -> Option<&ActorRef<ConnectionActor<Self>>>;
    fn get_blowfish(&self) -> &Encryption;

    /// # Errors
    /// - fail to write padding
    ///
    fn send_packet(&self, packet: impl SendablePacket) -> impl Future<Output = anyhow::Result<()>> {
        async move {
            let data = get_bytes(self.get_blowfish(), packet)?;
            send_packet_blocking(self.get_packet_sender(), data).await
        }
    }
    fn send_packet_no_wait(
        &self,
        packet: impl SendablePacket,
    ) -> impl Future<Output = anyhow::Result<()>> {
        async move {
            let data = get_bytes(self.get_blowfish(), packet)?;
            send_packet(self.get_packet_sender(), data).await
        }
    }
    fn send_delayed_packet(
        &self,
        packet: impl SendablePacket,
        delay: Duration,
    ) -> impl Future<Output = anyhow::Result<()>> {
        async move {
            let data = get_bytes(self.get_blowfish(), packet)?;
            send_delayed_packet(self.get_packet_sender(), data, delay).await
        }
    }
}
