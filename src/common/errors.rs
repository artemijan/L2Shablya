use log::error;
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum PacketErrors {
    #[error("Failed to encrypt message")]
    Encryption(#[from] anyhow::Error),
    #[error("Failed to write data to packet {max_size:?}")]
    PacketWrite { max_size: usize },
    #[error("Unable to encode string in format {0}")]
    #[allow(unused)]
    Encode(String),
    #[error("Unable to decrypt client packet")]
    DecryptBlowfishError,
    #[error("Unable to send packet")]
    #[allow(unused)]
    SendPacketError,
    #[error("Client packet not found, opcode is: {opcode:?}")]
    ClientPacketNotFound { opcode: usize },
    #[error("Unable to send init packet to the client")]
    UnableToSendInitPacket,
    #[error("Unable to handle client packet: {msg:?}")]
    UnableToHandleClientPacket { msg: String },
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum RsaErrors {
    #[error("Unable to read pem key")]
    ErrorReadingPem,
}
