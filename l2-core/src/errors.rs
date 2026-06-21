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

pub trait KameoAnyhowExt<T> {
    /// Converts a Kameo SendError into an anyhow::Error
    fn anyhow(self) -> anyhow::Result<T>;
}

impl<T, M, E> KameoAnyhowExt<T> for Result<T, kameo::error::SendError<M, E>>
where
    M: std::fmt::Debug,
    E: std::fmt::Debug,
{
    fn anyhow(self) -> anyhow::Result<T> {
        self.map_err(|e| anyhow::anyhow!("Actor communication error: {:?}", e))
    }
}
