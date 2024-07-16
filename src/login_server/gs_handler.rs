use crate::common::errors::PacketErrors;
use crate::common::message::Message;
use crate::crypt::{new_crypt::NewCrypt, rsa::ScrambledRSAKeyPair};
use crate::login_server::{controller::LoginController, PacketHandler};
use crate::packet::{
    common::write::SendablePacketBuffer, common::SendablePacket, error::PacketRunError,
    to_gs::InitLS, login_fail::LoginFail, gs_factory::build_gs_packet, LoginFailReasons,
};
use anyhow::{bail, Error};
use async_trait::async_trait;
use openssl::error::ErrorStack;
use sqlx::AnyPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, RwLock};

#[derive(Debug, Clone)]
pub struct GSHandler {
    tcp_reader: Arc<Mutex<OwnedReadHalf>>,
    tcp_writer: Arc<Mutex<OwnedWriteHalf>>,
    lc: Arc<LoginController>,
    db_pool: AnyPool,
    key_pair: ScrambledRSAKeyPair,
    blowfish: NewCrypt,
    pub connection_state: GSConnectionState,
    pub server_id: Option<u8>,
    unhandled_messages: Arc<RwLock<HashMap<String, Message>>>,
}

#[derive(Debug, Clone)]
pub enum GSConnectionState {
    Initial,
    Connected,
    BfConnected,
    Authed,
}

impl GSConnectionState {
    pub fn transition_to(
        &mut self,
        desired_state: &GSConnectionState,
    ) -> Result<Option<Box<dyn SendablePacket>>, PacketRunError> {
        match (&self, desired_state) {
            (Self::Initial, Self::Connected) => *self = Self::Connected,
            (Self::Connected, Self::BfConnected) => *self = Self::BfConnected,
            (Self::BfConnected, Self::Authed) => *self = Self::Authed,
            _ => {
                return Err(PacketRunError {
                    msg: Some(format!(
                        "Can not upgrade connection state for game server from {:?}, to {:?}",
                        self, desired_state
                    )),
                    response: Some(Box::new(LoginFail::new(LoginFailReasons::ReasonNotAuthed))),
                });
            }
        }
        Ok(None)
    }
}

impl GSHandler {
    const STATIC_KEY: &'static str = "_;v.]05-31!|+-%xT!^[$\0";
    pub fn new(mut stream: TcpStream, db_pool: AnyPool, lc: Arc<LoginController>) -> Self {
        let (tcp_reader, tcp_writer) = stream.into_split();
        let writer = Arc::new(Mutex::new(tcp_writer));
        let reader = Arc::new(Mutex::new(tcp_reader));
        GSHandler {
            tcp_reader: reader,
            tcp_writer: writer,
            db_pool,
            key_pair: lc.get_random_rsa_key_pair(),
            blowfish: NewCrypt::from_u8_key(Self::STATIC_KEY.as_bytes()),
            connection_state: GSConnectionState::Initial,
            lc,
            unhandled_messages: Arc::new(RwLock::new(HashMap::new())),
            server_id: None,
        }
    }

    pub fn start_channel(&self) {
        let (rx, mut tx) = mpsc::channel::<Message>(100);
        self.lc.connect_gs(self.server_id.unwrap(), rx);
        let mut messages = self.unhandled_messages.clone();
        //todo: we need to make GSHandler cloneable, so we can reuse the logic in different tasks;
        let mut gs_handler = self.clone();
        tokio::spawn(async move {
            loop {
                if let Some(t) = tx.recv().await {
                    let mut lock = messages.write().await;
                    //the message has been sent already, there is no sense to do it twice
                    if lock.contains_key(&t.id) {
                        let _ = t.response.send(None);
                    } else {
                        // send packet later, now we only remember it
                        let req_bytes = t.request.get_bytes();
                        if gs_handler.send_bytes(req_bytes).await.is_ok() {
                            lock.insert(t.id.clone(), t);
                        } else {
                            t.response.send(None).unwrap();
                        }
                    }
                }
            }
        });
    }
    pub async fn pop_unhandled_message(&self, key: &str) -> Option<Message> {
        let mut lock = self.unhandled_messages.write().await;
        lock.remove(key)
    }
    pub fn set_blowfish_key(&mut self, new_bf_key: Vec<u8>) {
        self.blowfish = NewCrypt::from_u8_key(&new_bf_key);
    }

    pub fn set_connection_state(&mut self, state: GSConnectionState) {
        self.connection_state = state
    }
    pub fn decrypt(&self, data: &mut [u8]) -> Result<(), PacketErrors> {
        self.blowfish.decrypt(data, 0, data.len())
    }

    pub fn decrypt_rsa(&self, data: &mut [u8]) -> Result<Vec<u8>, ErrorStack> {
        self.key_pair.decrypt_data(data)
    }
}

#[async_trait]
impl PacketHandler for GSHandler {
    fn get_handler_name() -> String {
        "Game server handler".to_string()
    }

    fn get_lc(&self) -> &Arc<LoginController> {
        &self.lc
    }
    async fn on_connect(&mut self) -> Result<(), PacketErrors> {
        println!(
            "Game server connected: {:?}",
            self.tcp_reader.lock().await.peer_addr().unwrap()
        );
        self.connection_state = GSConnectionState::Connected;
        let init_packet = Box::new(InitLS::new(self.key_pair.get_modulus()));
        self.send_packet(init_packet).await?;
        Ok(())
    }

    fn get_stream_reader_mut(&mut self) -> &Arc<Mutex<OwnedReadHalf>> {
        &self.tcp_reader
    }
    async fn get_stream_writer_mut(&self) -> &Arc<Mutex<OwnedWriteHalf>> {
        &self.tcp_writer
    }

    fn get_timeout(&self) -> Option<u64> {
        None
    }

    async fn send_packet(&mut self, packet: Box<dyn SendablePacket>) -> Result<(), Error> {
        self.send_bytes(packet.get_bytes()).await
    }
    async fn send_bytes(&self, bytes: Vec<u8>) -> Result<(), Error> {
        let mut packet_buffer = SendablePacketBuffer::from_bytes(&bytes);
        packet_buffer.write_i32(0).unwrap();
        let padding = (packet_buffer.get_size() - 2) % 8;
        if padding != 0 {
            for _ in padding..8 {
                packet_buffer.write_u8(0)?;
            }
        }
        let mut data_vec = packet_buffer.get_data();
        let size = data_vec.len() - 2;
        NewCrypt::append_checksum(&mut data_vec, 2, size);
        self.blowfish.crypt(&mut data_vec, 2, size);
        self.get_stream_writer_mut()
            .await
            .lock()
            .await
            .write_all(&data_vec)
            .await?;
        // self.tcp_writer.lock().unwrap().write_all(&data_vec).await?;
        Ok(())
    }

    async fn on_receive_bytes(
        &mut self,
        packet_size: usize,
        bytes: &mut [u8],
    ) -> Result<(), Error> {
        self.blowfish.decrypt(bytes, 0, packet_size)?;
        if !NewCrypt::verify_checksum(bytes, 0, packet_size) {
            bail!("Can not verify check sum.")
        }
        let handler = build_gs_packet(bytes).ok_or_else(|| PacketErrors::ClientPacketNotFound {
            opcode: bytes[0] as usize,
        })?;
        let resp = handler.handle(self).await;
        self.handle_result(resp).await
    }
}
