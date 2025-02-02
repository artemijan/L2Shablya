use crate::errors::Packet;
use anyhow::Result as Res;
use bytes::{BufMut, BytesMut};
use encoding::all::UTF_16LE;
use encoding::{EncoderTrap, Encoding};

#[derive(Debug, Clone, Default)]
pub struct SendablePacketBuffer {
    data: BytesMut,
}

#[allow(unused, clippy::cast_possible_truncation, clippy::missing_errors_doc)]
impl SendablePacketBuffer {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            data: BytesMut::with_capacity(0),
        }
    }
    #[must_use]
    pub fn new() -> Self {
        let mut data = BytesMut::with_capacity(256);
        data.put_i16_le(0); // first 2 bytes are reserved for storing packet length
        Self { data }
    }

    pub fn write(&mut self, value: u8) -> Res<()> {
        self.data.put_u8(value);
        Ok(())
    }
    pub fn write_bytes(&mut self, value: &[u8]) -> Res<()> {
        self.data.put_slice(value);
        Ok(())
    }
    pub fn write_c_utf16le_string(&mut self, value: Option<&str>) -> Res<()> {
        if let Some(st) = value {
            self.write_bytes(
                &UTF_16LE
                    .encode(st, EncoderTrap::Strict)
                    .map_err(|_| Packet::Encode("UTF_16LE".to_owned()))?,
            )?;
        }
        self.write_i16(0) //null char for C-like strings
    }
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn write_sized_c_utf16le_string(&mut self, value: Option<&str>) -> Res<()> {
        if let Some(st) = value {
            self.write_i16((st.len() & 0xff) as i16)?;
            self.write_bytes(
                &UTF_16LE
                    .encode(st, EncoderTrap::Strict)
                    .map_err(|_| Packet::Encode("UTF_16LE".to_owned()))?,
            )?;
        } else {
            self.write_i16(0)?; // null char for C-like strings
        }
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn write_i8(&mut self, value: i8) -> Res<()> {
        self.write(value as u8)
    }

    pub fn write_u8(&mut self, value: u8) -> Res<()> {
        self.write(value)
    }

    pub fn write_bool(&mut self, value: bool) -> Res<()> {
        self.write_u8(u8::from(value))
    }

    pub fn write_i16(&mut self, value: i16) -> Res<()> {
        self.data.put_i16_le(value);
        Ok(())
    }

    pub fn write_u16(&mut self, value: u16) -> Res<()> {
        self.data.put_u16_le(value);
        Ok(())
    }
    pub fn write_i16_from_bool(&mut self, value: bool) -> Res<()> {
        self.write_i16(i16::from(value))
    }

    pub fn write_i32(&mut self, value: i32) -> Res<()> {
        self.data.put_i32_le(value);
        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> Res<()> {
        self.data.put_u32_le(value);
        Ok(())
    }
    pub fn write_i32_from_bool(&mut self, value: bool) -> Res<()> {
        self.write_i32(i32::from(value))
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn write_i64(&mut self, value: i64) -> Res<()> {
        self.data.put_i64_le(value);
        Ok(())
    }
    pub fn write_i64_from_bool(&mut self, value: bool) -> Res<()> {
        self.write_i64(i64::from(value))
    }

    pub fn write_f32(&mut self, value: f32) -> Res<()> {
        self.data.put_f32_le(value);
        Ok(())
    }

    pub fn write_f64(&mut self, value: f64) -> Res<()> {
        self.data.put_f64_le(value);
        Ok(())
    }

    #[must_use]
    pub fn get_size(&self) -> usize {
        self.data.len() // also substract first 2 bytes to store the length
    }

    pub fn get_data_mut(&mut self, with_padding: bool) -> &mut [u8] {
        // Add size info at start (unsigned short - max size 65535).
        if with_padding {
            self.write_padding();
        }
        self.write_packet_size();
        self.data.as_mut()
    }

    pub fn write_packet_size(&mut self) {
        let size = self.get_size();
        self.data[0] = (size & 0xff) as u8;
        self.data[1] = ((size >> 8) & 0xffff) as u8;
    }

    pub fn write_padding(&mut self) -> Res<()> {
        self.write_i32(0)?;
        let padding = (self.data.len() - 2) % 8;
        if padding != 0 {
            for _ in padding..8 {
                self.write_u8(0)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_write_packet() {
        let mut packet = SendablePacketBuffer::new();
        packet.write_c_utf16le_string(Some("Test")).unwrap();
        packet.write_sized_c_utf16le_string(Some("Test")).unwrap();
        packet.write(0).unwrap();
        packet.write_bool(true).unwrap();
        packet.write_i8(11).unwrap();
        packet.write_i16(11).unwrap();
        packet.write_i32(11).unwrap();
        packet.write_i64(11).unwrap();
        packet.write_i16_from_bool(true).unwrap();
        packet.write_i32_from_bool(true).unwrap();
        packet.write_i64_from_bool(true).unwrap();
        packet.write_u8(0).unwrap();
        packet.write_u16(0).unwrap();
        packet.write_u32(0).unwrap();
        packet.write_bytes(&[0, 1, 2, 3]).unwrap();
        packet.write_f32(3.45).unwrap();
        packet.write_f64(3.45).unwrap();
        let bytes = packet.get_data_mut(true);
        assert_eq!(
            bytes,
            &[
                82, 0, 84, 0, 101, 0, 115, 0, 116, 0, 0, 0, 4, 0, 84, 0, 101, 0, 115, 0, 116, 0, 0,
                1, 11, 11, 0, 11, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 205, 204, 92, 64, 154, 153, 153, 153,
                153, 153, 11, 64, 0, 0, 0, 0, 0, 0
            ]
        );
    }
}
