use crate::common::packets::write::SendablePacketBuffer;
use error::PacketRun;
use std::fmt::Debug;

pub mod error;
pub mod read;
pub mod write;

pub type PacketResult = Result<Option<Box<dyn SendablePacket>>, PacketRun>;

pub trait SendablePacket: Debug + Send + Sync {
    fn get_bytes_mut(&mut self) -> &mut [u8] {
        let buff = self.get_buffer_mut();
        buff.get_data_mut()
    }
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer;
}

pub trait ReadablePacket: Debug + Send + Sync {
    fn read(data: &[u8]) -> Option<Self>
    where
        Self: Sized + ReadablePacket;
}
