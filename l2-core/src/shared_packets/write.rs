use crate::errors::Packet;
use anyhow::Result as Res;
use bytes::{BufMut, Bytes, BytesMut};
use encoding::all::UTF_16LE;
use encoding::{EncoderTrap, Encoding};
use std::fmt;

#[derive(Clone, Default)]
pub struct SendablePacketBuffer {
    data: BytesMut,
}
impl fmt::Debug for SendablePacketBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SendablePacketBuffer")
            .finish()
    }
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

    pub fn write<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<u8>,
    {
        self.data.put_u8(value.into());
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
        self.write_i16(0i16) //null char for C-like strings
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
            self.write_i16(0i16)?; // null char for C-like strings
        }
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn write_i8<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<i8>,
    {
        self.write(value.into() as u8)
    }

    pub fn write_u8<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<u8>,
    {
        self.write(value.into())
    }

    pub fn write_bool<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<bool>,
    {
        self.write_u8(u8::from(value.into()))
    }

    pub fn write_i16<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<i16>,
    {
        self.data.put_i16_le(value.into());
        Ok(())
    }

    pub fn write_u16<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<u16>,
    {
        self.data.put_u16_le(value.into());
        Ok(())
    }

    pub fn write_i32<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<i32>,
    {
        self.data.put_i32_le(value.into());
        Ok(())
    }

    pub fn write_u32<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<u32>,
    {
        self.data.put_u32_le(value.into());
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn write_i64<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<i64>,
    {
        self.data.put_i64_le(value.into());
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn write_u64<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<u64>,
    {
        self.data.put_u64_le(value.into());
        Ok(())
    }

    pub fn write_f32<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<f32>,
    {
        self.data.put_f32_le(value.into());
        Ok(())
    }

    pub fn write_f64<T>(&mut self, value: T) -> Res<()>
    where
        T: Into<f64>,
    {
        self.data.put_f64_le(value.into());
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
    #[must_use]
    pub fn freeze(mut self, with_padding: bool) -> Bytes {
        if with_padding {
            self.write_padding();
        }
        self.write_packet_size();
        self.data.freeze()
    }

    #[must_use]
    pub fn take(mut self) -> BytesMut {
        self.write_packet_size();
        self.data
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
        packet.write_i16(11i16).unwrap();
        packet.write_i32(11).unwrap();
        packet.write_i64(11).unwrap();
        packet.write_i16(true).unwrap();
        packet.write_i32(true).unwrap();
        packet.write_i64(true).unwrap();
        packet.write_u8(0).unwrap();
        packet.write_u16(0u16).unwrap();
        packet.write_u32(0u32).unwrap();
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
