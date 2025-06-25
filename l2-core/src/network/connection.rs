use anyhow::{anyhow, bail};
use bytes::{Bytes, BytesMut};
use kameo::actor::{ActorRef, WeakActorRef};
use kameo::error::ActorStopReason;
use kameo::message::{Context, Message};
use kameo::Actor;
use log::warn;
use std::fmt;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{mpsc, watch};
use tokio::time::{sleep, timeout};
use tracing::{error, info};

pub const PACKET_SIZE_BYTES: usize = 2;

pub struct HandleIncomingPacket(pub BytesMut);

impl fmt::Debug for HandleIncomingPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HandleIncomingPacket").finish()
    }
}
pub struct SendPacket {
    pub data: Bytes,
    pub with_delay: Option<Duration>,
}

pub struct ConnectionActor<T>
where
    T: Actor + Message<HandleIncomingPacket>,
{
    addr: Ipv4Addr,
    receiver: ActorRef<T>,
    timeout: Duration,
    writer: Option<Box<dyn AsyncWrite + Send + Unpin>>,
    reader: Option<Box<dyn AsyncRead + Send + Unpin>>,
    reader_handle: Option<tokio::task::JoinHandle<()>>,
    writer_handle: Option<tokio::task::JoinHandle<()>>,
    packet_sender: Option<UnboundedSender<Bytes>>,
    shutdown_trigger: Option<watch::Sender<bool>>,
}

impl<T> ConnectionActor<T>
where
    T: Actor + Message<HandleIncomingPacket>,
{
    #[must_use = "Actor must be started to be used"]
    pub fn new(
        receiver: ActorRef<T>,
        addr: Ipv4Addr,
        reader: Box<dyn AsyncRead + Send + Unpin>,
        writer: Box<dyn AsyncWrite + Send + Unpin>,
        timeout: Duration,
    ) -> Self {
        Self {
            addr,
            receiver,
            timeout,
            writer: Some(writer),
            reader: Some(reader),
            packet_sender: None,
            reader_handle: None,
            writer_handle: None,
            shutdown_trigger: None,
        }
    }
    async fn read_packet(
        socket: &mut (impl AsyncRead + Send + Unpin),
    ) -> anyhow::Result<(usize, BytesMut)> {
        let mut size_buf = [0; PACKET_SIZE_BYTES];
        if socket.read_exact(&mut size_buf).await.is_err() {
            // at this stage, the client wanted to disconnect
            return Ok((0, BytesMut::new()));
        }
        let defined_size = u16::from_le_bytes(size_buf) as usize;
        if defined_size < PACKET_SIZE_BYTES {
            bail!("Packet size is too low, it's not expected: {defined_size}");
        }

        let size = defined_size - PACKET_SIZE_BYTES;
        // Read the body of the packet based on the size
        let mut body = BytesMut::with_capacity(size);
        body.resize(size, 0);
        socket.read_exact(&mut body).await?;
        Ok((size, body))
    }
}

impl<T> Actor for ConnectionActor<T>
where
    T: Actor + Message<HandleIncomingPacket>,
{
    type Args = Self;
    type Error = anyhow::Error;
    async fn on_start(mut state: Self::Args, actor_ref: ActorRef<Self>) -> anyhow::Result<Self> {
        let mut reader = state.reader.take().expect("Reader already taken");
        let (tx, mut rx): (UnboundedSender<Bytes>, UnboundedReceiver<Bytes>) =
            mpsc::unbounded_channel();
        let mut write_half = state.writer.take().expect("Writer already taken");
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let mut writer_shutdown_rx = shutdown_rx.clone();
        let mut reader_shutdown_rx = shutdown_rx.clone();
        state.shutdown_trigger = Some(shutdown_tx);
        let actor = actor_ref.clone();
        state.writer_handle = Some(tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = writer_shutdown_rx.changed() => {
                        if *writer_shutdown_rx.borrow() {
                            break;
                        }
                    }

                    Some(data) = rx.recv() => {
                        if let Err(e) = write_half.write_all(&data).await {
                            error!("Writer error: {e:?}");
                            break;
                        }
                    }
                    else => break,
                }
            }
            let _ = write_half.shutdown().await;
            if actor.is_alive(){
                let _ = actor.stop_gracefully().await;
                actor.wait_for_shutdown().await;
            }
        }));
        let receiver_addr = state.receiver.clone();

        if state.timeout.is_zero() {
            state.timeout = Duration::from_secs(u64::MAX);
        }
        let read_timeout = state.timeout;
        let ip = state.addr;
        let actor = actor_ref.clone();
        state.reader_handle = Some(tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = reader_shutdown_rx.changed()=>{
                        if *reader_shutdown_rx.borrow() {
                            break;
                        }
                    }
                    result = timeout(read_timeout, Self::read_packet(&mut reader)) => {
                        match result {
                            Err(_) => {
                                info!("Connection {ip} read timeout, elapsed: {read_timeout:?}");
                                break;
                            }
                            Ok(Err(_) | Ok((0, _))) => {
                                info!("Connection {ip} closed by client");
                                break;
                            }
                            Ok(Ok((_, data))) => {
                                let _ = receiver_addr.tell(HandleIncomingPacket(data)).await;
                            }
                        }
                    }
                }
            }
            if actor.is_alive(){
                let _ = actor.stop_gracefully().await;
                actor.wait_for_shutdown().await;
            }
        }));
        state.packet_sender = Some(tx);
        Ok(state)
    }
    async fn on_stop(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        _reason: ActorStopReason,
    ) -> Result<(), Self::Error> {
        if let Some(shutdown) = &self.shutdown_trigger {
            let _ = shutdown.send(true);
        }
        info!("[Connection {}] closed", self.addr);
        Ok(())
    }
}

impl<T> Message<SendPacket> for ConnectionActor<T>
where
    T: Actor + Message<HandleIncomingPacket>,
{
    type Reply = anyhow::Result<()>;
    async fn handle(
        &mut self,
        msg: SendPacket,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Some(ps) = self.packet_sender.as_ref() {
            if let Some(dur) = msg.with_delay {
                let sender = ps.clone();
                tokio::spawn(async move {
                    sleep(dur).await;
                    // we are okay to ignore the error because no one waits for a result
                    let _ = sender.send(msg.data);
                });
            } else {
                ps.send(msg.data)?;
            }
        } else {
            bail!("Packet sender not initialized");
        }
        Ok(())
    }
}

///
/// # Errors
/// - when an actor can't receive the message
/// - where there is no sender
pub async fn send_packet<T>(
    sender: Option<&ActorRef<ConnectionActor<T>>>,
    packet: Bytes,
) -> anyhow::Result<()>
where
    T: Actor + Message<HandleIncomingPacket>,
{
    if let Some(ps) = sender {
        ps.tell(SendPacket {
            data: packet,
            with_delay: None,
        })
        .await
        .map_err(|e| anyhow!("Error sending packet: {e:?}"))
    } else {
        bail!("No packet sender set, probably connection is not ready yet.")
    }
}

///
/// # Errors
/// - when an actor can't receive the message
/// - where there is no sender
pub async fn send_packet_blocking<T>(
    sender: Option<&ActorRef<ConnectionActor<T>>>,
    packet: Bytes,
) -> anyhow::Result<()>
where
    T: Actor + Message<HandleIncomingPacket>,
{
    if let Some(ps) = sender {
        let reply = ps
            .ask(SendPacket {
                data: packet,
                with_delay: None,
            })
            .await;
        match reply {
            Ok(_) => Ok(()),
            Err(e) => bail!("Error sending packet: {e:?}"),
        }
    } else {
        bail!("No packet sender set, probably connection is not ready yet.")
    }
}

///
/// # Errors
/// - when an actor can't receive the message
/// - where there is no sender
pub async fn send_delayed_packet<T>(
    sender: Option<&ActorRef<ConnectionActor<T>>>,
    packet: Bytes,
    delay: Duration,
) -> anyhow::Result<()>
where
    T: Actor + Message<HandleIncomingPacket>,
{
    if let Some(ps) = sender {
        ps.tell(SendPacket {
            data: packet,
            with_delay: Some(delay),
        })
        .await
        .map_err(|e| anyhow!("Error sending packet: {e:?}"))
    } else {
        bail!("No packet sender set, probably connection is not ready yet.")
    }
}
