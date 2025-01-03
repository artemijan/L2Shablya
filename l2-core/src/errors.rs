use log::error;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
#[allow(unused)]
pub enum Packet {
    #[error("Failed to encrypt message")]
    Encryption(#[from] anyhow::Error),
    #[error("Failed to write data to packet {max_size:?}")]
    Write { max_size: usize },
    #[error("Unable to encode string in format {0}")]
    Encode(String),
    #[error("Unable to decrypt client packet")]
    DecryptBlowfishError,
    #[error("Unable to send packet")]
    SendPacketError,
    #[error("Client packet not found, opcode is: {opcode:?}")]
    ClientPacketNotFound { opcode: usize },
    #[error("Unable to send init packet to the client")]
    UnableToSendInit,
    #[error("Unable to handle client packet: {msg:?}")]
    UnableToHandleClient { msg: String },
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Rsa {
    #[error("Unable to read pem key")]
    ErrorReadingPem,
}
