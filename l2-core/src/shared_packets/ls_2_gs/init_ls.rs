use crate as l2_core;
use crate::constants;
use crate::shared_packets::common::{LoginServerOpcodes, ReadablePacket};
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use macro_common::SendablePacketImpl;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct InitLS {
    buffer: SendablePacketBuffer,
    pub revision: i32,
    pub public_key: Vec<u8>,
}

impl InitLS {
    #[must_use]
    pub fn new(public_key: Vec<u8>) -> Self {
        let mut init_ls = InitLS {
            buffer: SendablePacketBuffer::new(),
            revision: constants::PROTOCOL_REVISION,
            public_key,
        };
        let _ = init_ls.write_all();
        init_ls
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(LoginServerOpcodes::Init as u8)?;
        self.buffer.write_i32(self.revision)?; // LS protocol revision
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        self.buffer.write_i32(self.public_key.len() as i32)?; // key length
        self.buffer.write_bytes(self.public_key.as_slice())?; // RSA Public Key
        Ok(())
    }
}

impl ReadablePacket for InitLS {
    const PACKET_ID: u8 = 0x00;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let _packet_id = buffer.read_byte()?;
        let revision = buffer.read_i32()?; // LS protocol revision
        let key_size = buffer.read_i32()?; // key length
        #[allow(clippy::cast_sign_loss)]
        let public_key = buffer.read_bytes(key_size as usize)?.to_vec(); // RSA Public Key
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            revision,
            public_key,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{constants, shared_packets::{common::ReadablePacket, ls_2_gs::InitLS}};

    #[test]
    fn test_read() {
        let data = [
            0, 12, 0, 0, 0, 16, 0, 0, 0, 16, 17, 18, 19, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
            31, 32,
        ];
        let packet = InitLS::read(&data).unwrap();
        assert_eq!(packet.revision, 12);
        assert_eq!(packet.public_key.len(), 16);
        assert_eq!(
            packet.public_key,
            [16, 17, 18, 19, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]
        );
    }
    #[test]
    fn test_new() {
        let pub_key = vec![
            16, 17, 18, 19, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let packet = InitLS::new(pub_key);
        // assert_eq!(packet.revision, 0);
        assert_eq!(packet.revision, constants::PROTOCOL_REVISION);
        assert_eq!(packet.public_key.len(), 16);
        assert_eq!(
            packet.public_key,
            [16, 17, 18, 19, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32]
        );
    }
}
