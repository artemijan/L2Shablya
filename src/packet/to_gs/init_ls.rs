use crate::packet::common::write::SendablePacketBuffer;
use crate::packet::common::SendablePacket;
use crate::packet::LoginServerOpcodes;

#[derive(Debug)]
pub struct InitLS {
    pub buffer: SendablePacketBuffer,
    public_key: Vec<u8>,
}

impl InitLS {
    pub const PROTOCOL_REVISION: i32 = 0x0106;
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
        self.buffer.write_i32(Self::PROTOCOL_REVISION)?; // LS protocol revision
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
