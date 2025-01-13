use async_trait::async_trait;
use l2_core::config::gs::GSServer;
use l2_core::shared_packets::common::SendablePacket;
use l2_core::shared_packets::write::SendablePacketBuffer;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct ProtocolResponse {
    buffer: SendablePacketBuffer,
    is_protocol_ok: bool,
}

impl ProtocolResponse {
    const PACKET_ID:u8 = 0x2E;
    pub fn new(key:&[u8], is_protocol_ok:bool, cfg:&GSServer) -> anyhow::Result<ProtocolResponse> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_bool(is_protocol_ok)?;
        buffer.write_bytes(&key[0..8])?; // 8 bytes
        buffer.write_u32(u32::from(cfg.enable_encryption))?; // 0 encryption disabled | 1 enabled
        buffer.write_u32(u32::from(cfg.server_id))?;
        buffer.write(1)?; // ???
        buffer.write_u32(0)?; // obfuscation key
        buffer.write(1)?; // is_classic
        Ok(ProtocolResponse {
            buffer,
            is_protocol_ok
        })
    }
    pub fn fail(cfg:&GSServer)->anyhow::Result<ProtocolResponse> {
        let mut buffer = SendablePacketBuffer::new();
        buffer.write(Self::PACKET_ID)?;
        buffer.write_bool(false)?;
        buffer.write_bytes(&[0;8])?; // 8 bytes
        buffer.write_u32(u32::from(cfg.enable_encryption))?; // 0 encryption disabled | 1 enabled
        buffer.write_u32(u32::from(cfg.server_id))?;
        buffer.write(1)?; // ???
        buffer.write_u32(0)?; // obfuscation key
        buffer.write(1)?; // is_classic
        Ok(ProtocolResponse {
            buffer,
            is_protocol_ok:false
        })
    }
}

#[async_trait]
impl SendablePacket for ProtocolResponse {
    fn get_buffer_mut(&mut self) -> &mut SendablePacketBuffer {
        &mut self.buffer
    }
}
