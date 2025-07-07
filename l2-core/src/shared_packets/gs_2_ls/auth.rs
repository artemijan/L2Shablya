use bytes::BytesMut;
use crate::config::gs::GSServerConfig;
use crate::shared_packets::common::ReadablePacket;
use crate::shared_packets::read::ReadablePacketBuffer;
use crate::shared_packets::write::SendablePacketBuffer;
use num_traits::ToBytes;
use macro_common::SendablePacket;
use crate as l2_core;

#[derive(Clone, Debug, Default, SendablePacket)]
pub struct RequestAuthGS {
    pub buffer: SendablePacketBuffer,
    pub desired_id: u8,
    pub accept_alternative_id: bool,
    pub host_reserved: bool,
    pub port: u16,
    pub max_players: u32,
    pub hex_id: Vec<u8>,
    pub hosts: Vec<String>,
}

impl RequestAuthGS {
    ///
    /// # Errors
    /// - if writing packet bytes is not possible, for example when packet size is too big
    pub fn new(cfg: &GSServerConfig) -> anyhow::Result<Self> {
        let mut inst = Self {
            buffer: SendablePacketBuffer::new(),
            desired_id: cfg.server_id,
            accept_alternative_id: cfg.accept_alternative_id,
            host_reserved: cfg.host_reserved,
            port: cfg.listeners.clients.connection.port,
            max_players: cfg.max_players,
            hex_id: cfg.hex_id.to_be_bytes(),
            hosts: cfg.get_hosts(),
        };
        inst.write_all()?;
        Ok(inst)
    }
    #[allow(clippy::cast_possible_truncation)]
    fn write_all(&mut self) -> Result<(), anyhow::Error> {
        self.buffer.write(0x01)?; // packet id
        self.buffer.write(self.desired_id)?;
        self.buffer.write_bool(self.accept_alternative_id)?;
        self.buffer.write_bool(self.host_reserved)?;
        self.buffer.write_u16(self.port)?;
        self.buffer.write_u32(self.max_players)?;
        self.buffer.write_u32(self.hex_id.len() as u32)?;
        self.buffer.write_bytes(&self.hex_id)?;
        self.buffer.write_u32((self.hosts.len() / 2) as u32)?; // we cut it by half because it's actually a pair of network/ip
        for h in &self.hosts {
            self.buffer.write_c_utf16le_string(Some(h))?;
        }
        Ok(())
    }
}

#[allow(clippy::cast_sign_loss)]
impl ReadablePacket for RequestAuthGS {
    const PACKET_ID: u8 = 0x01;
    const EX_PACKET_ID: Option<u16> = None;

    fn read(data: BytesMut) -> anyhow::Result<Self> {
        let mut buffer: ReadablePacketBuffer = ReadablePacketBuffer::new(data);
        buffer.read_byte()?;
        let desired_id = buffer.read_byte()?;
        let accept_alternative_id = buffer.read_boolean()?;
        let host_reserved = buffer.read_boolean()?;
        let port = buffer.read_u16()?;
        let max_players = buffer.read_u32()?;
        let mut size = buffer.read_u32()?;
        let hex_id = buffer.read_bytes(size as usize)?.to_vec();
        size = buffer.read_u32()? * 2;
        let hosts = buffer.read_n_strings(size as usize)?;
        Ok(Self {
            buffer: SendablePacketBuffer::empty(),
            desired_id,
            accept_alternative_id,
            host_reserved,
            port,
            max_players,
            hex_id,
            hosts,
        })
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use crate::shared_packets::common::ReadablePacket;
    use crate::shared_packets::gs_2_ls::RequestAuthGS;
    use crate::shared_packets::write::SendablePacketBuffer;
    fn get_bytes() -> [u8; 74] {
        [
            74, 0, 1, 3, 1, 0, 58, 8, 57, 27, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 49, 0, 50, 0, 55, 0,
            46, 0, 48, 0, 46, 0, 48, 0, 46, 0, 48, 0, 47, 0, 48, 0, 0, 0, 49, 0, 50, 0, 55, 0, 46,
            0, 48, 0, 46, 0, 48, 0, 46, 0, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]
    }
    #[test]
    fn test_instantiate() {
        let mut p = RequestAuthGS {
            buffer: SendablePacketBuffer::new(),
            desired_id: 3,
            accept_alternative_id: true,
            host_reserved: false,
            port: 2106,
            max_players: 6969,
            hex_id: vec![],
            hosts: vec!["127.0.0.0/0".to_string(), "127.0.0.1".to_string()],
        };
        p.write_all().unwrap();
        p.buffer.write_padding().unwrap();
        let bytes = p.buffer.take().to_vec();
        assert_eq!(bytes.len(), 74);
        assert_eq!(bytes, get_bytes());
    }
    #[test]
    fn test_read() {
        let p = RequestAuthGS::read(BytesMut::from(&get_bytes()[2..])).unwrap();
        assert_eq!(p.desired_id, 3);
        assert!(p.accept_alternative_id);
        assert!(!p.host_reserved);
        assert_eq!(p.port, 2106);
        assert_eq!(p.max_players, 6969);
        assert!(p.hex_id.is_empty());
        assert_eq!(
            p.hosts,
            vec!["127.0.0.0/0".to_string(), "127.0.0.1".to_string()]
        );
    }
}
