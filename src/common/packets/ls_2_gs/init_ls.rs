use crate::common::constants;
use crate::common::packets::common::{LoginServerOpcodes, ReadablePacket, SendablePacket};
use crate::common::packets::write::SendablePacketBuffer;
use crate::common::packets::read::ReadablePacketBuffer;

#[derive(Debug)]
pub struct InitLS {
    buffer: SendablePacketBuffer,
    public_key: Vec<u8>,
}

impl InitLS {
    pub fn new(public_key: Vec<u8>) -> Self {
        let mut init_ls = InitLS {
            buffer: SendablePacketBuffer::new(),
            public_key,
        };
        let _ = init_ls.write_all();
        init_ls
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(LoginServerOpcodes::Init as u8)?;
        self.buffer.write_i32(constants::PROTOCOL_REVISION)?; // LS protocol revision
        #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
        self.buffer.write_i32(self.public_key.len() as i32)?; // key length
        self.buffer.write_bytes(self.public_key.as_slice())?; // RSA Public Key
        Ok(())
    }
}

impl SendablePacket for InitLS {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}

impl ReadablePacket for InitLS{
    fn read(data: &[u8]) -> Option<Self> {
        let mut buffer = ReadablePacketBuffer::new(data.to_vec());
        let packet_id = buffer.read_byte();
        let revision = buffer.read_i32(); // LS protocol revision
        let key_size = buffer.read_i32(); // key length
        #[allow(clippy::cast_sign_loss)]
        let public_key = buffer.read_bytes(key_size as usize); // RSA Public Key
        Some(Self{
            buffer: SendablePacketBuffer::empty(),
            public_key
        })
    }
}
