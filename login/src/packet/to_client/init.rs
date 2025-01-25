use l2_core::shared_packets::{
    common::{LoginServerOpcodes, SendablePacket},
    write::SendablePacketBuffer,
};
use macro_common::SendablePacketImpl;

#[derive(Debug, SendablePacketImpl)]
pub struct Init {
    pub buffer: SendablePacketBuffer,
    session_id: i32,
    public_key: Vec<u8>,
    blowfish_key: Vec<u8>,
}

impl Init {
    pub fn new(session_id: i32, public_key: Vec<u8>, blowfish_key: Vec<u8>) -> Init {
        let mut init = Init {
            buffer: SendablePacketBuffer::new(),
            session_id,
            public_key,
            blowfish_key,
        };
        let _ = init.write_all();
        init
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_i8(LoginServerOpcodes::Init as i8)?;
        self.buffer.write_i32(self.session_id)?; // session id
        self.buffer.write_i32(0x0000_c621)?; // protocol revision
        self.buffer.write_bytes(self.public_key.as_slice())?; // RSA Public Key
                                                              // unk GG related?
        self.buffer.write_i32(0x29DD_954E)?;
        self.buffer.write_i32(0x77C3_9CFC)?;
        #[allow(clippy::cast_possible_wrap)]
        self.buffer.write_i32(0x97AD_B620_u32 as i32)?; // 0x97ADB620 this overflows i32
        self.buffer.write_i32(0x07BD_E0F7)?;
        self.buffer.write_bytes(self.blowfish_key.as_slice())?; // BlowFish key
        self.buffer.write(0)?; // null termination ;)
        Ok(())
    }
}
