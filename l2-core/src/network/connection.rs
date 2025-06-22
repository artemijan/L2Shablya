use anyhow::{anyhow, bail};
use bytes::{Bytes, BytesMut};
use kameo::actor::{ActorRef, WeakActorRef};
use kameo::error::ActorStopReason;
use kameo::message::{Context, Message};
use kameo::Actor;
use std::fmt;
use std::net::Ipv4Addr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::{sleep, timeout};
use tracing::{error, info};

pub const PACKET_SIZE_BYTES: usize = 2;

pub struct Shutdown;

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

async fn stop_actor<T: Actor>(actor_ref: ActorRef<T>) {
    actor_ref
        .stop_gracefully()
        .await
        .unwrap_or_else(|_| panic!("Failed to stop Actor {actor_ref:?}"));
    actor_ref.wait_for_shutdown().await;
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
        let receiver_addr = state.receiver.clone();
        let self_addr = actor_ref.clone();

        state.writer_handle = Some(tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = write_half.write_all(&data).await {
                    error!("Writer error: {e:?}");
                    break;
                }
            }
            write_half.shutdown().await.unwrap();
            stop_actor(receiver_addr).await;
            stop_actor(self_addr).await;
        }));
        let receiver_addr = state.receiver.clone();
        let self_addr = actor_ref.clone();

        if state.timeout.is_zero() {
            state.timeout = Duration::from_secs(u64::MAX);
        }
        let read_timeout = state.timeout;
        let ip = state.addr;
        state.reader_handle = Some(tokio::spawn(async move {
            loop {
                match timeout(read_timeout, Self::read_packet(&mut reader)).await {
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
            stop_actor(receiver_addr).await;
            stop_actor(self_addr).await;
        }));
        state.packet_sender = Some(tx);
        Ok(state)
    }
    async fn on_stop(
        &mut self,
        _actor_ref: WeakActorRef<Self>,
        _reason: ActorStopReason,
    ) -> Result<(), Self::Error> {
        if let Some(reader) = self.reader_handle.as_ref() {
            reader.abort();
        }
        if let Some(writer) = self.writer_handle.as_ref() {
            writer.abort();
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
