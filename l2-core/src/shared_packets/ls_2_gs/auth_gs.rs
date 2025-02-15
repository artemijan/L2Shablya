use crate as l2_core;
use crate::shared_packets::{
    common::ReadablePacket, read::ReadablePacketBuffer, write::SendablePacketBuffer,
};
use macro_common::SendablePacketImpl;

#[derive(Debug, Clone, SendablePacketImpl)]
pub struct AuthGS {
    buffer: SendablePacketBuffer,
    pub server_id: u8,
    pub server_name: String,
}

impl AuthGS {
    #[must_use]
    pub fn new(server_id: u8, server_name: String) -> AuthGS {
        let mut gg = AuthGS {
            buffer: SendablePacketBuffer::new(),
            server_id,
            server_name,
        };
        let _ = gg.write_all();
        gg
    }
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write_u8(0x02)?;
        self.buffer.write_u8(self.server_id)?;
        self.buffer
            .write_c_utf16le_string(Some(&self.server_name))?;
        Ok(())
    }
}

impl ReadablePacket for AuthGS {
    const PACKET_ID: u8 = 0x02;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: &[u8]) -> anyhow::Result<Self> {
        let mut buffer = ReadablePacketBuffer::new(data);
        let _packet_id = buffer.read_byte()?;
        let server_id = buffer.read_byte()?;
        let server_name = buffer.read_c_utf16le_string()?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            server_id,
            server_name,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::shared_packets::{common::ReadablePacket, ls_2_gs::AuthGS};

    #[test]
    fn test_read() {
        let data = [2, 1, 116, 0, 101, 0, 115, 0, 116, 0, 0, 0];
        let packet = AuthGS::read(&data).unwrap();
        assert_eq!(packet.server_id, 1);
        assert_eq!(packet.server_name, "test");
    }
    #[test]
    fn test_new() {
        let expected = [14,0,2, 8, 115, 0, 101, 0, 114, 0, 118, 0, 0, 0];
        let mut packet = AuthGS::new(8, "serv".to_string());
        let data = packet.buffer.get_data_mut(false);
        assert_eq!(data, expected);
    }
}
