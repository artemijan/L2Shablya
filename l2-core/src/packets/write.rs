use crate::errors::Packet;
use anyhow::Result as Res;
use encoding::all::UTF_16LE;
use encoding::{EncoderTrap, Encoding};

#[derive(Debug, Clone, Default)]
pub struct SendablePacketBuffer {
    data: Vec<u8>,
    position: usize,
}

#[allow(unused, clippy::cast_possible_truncation, clippy::missing_errors_doc)]
impl SendablePacketBuffer {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            data: vec![0; 2],
            position: 2,
        }
    }
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: vec![0; 512],
            position: 2,
        }
    }
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            position: data.len(),
        }
    }
    pub fn write(&mut self, value: u8) -> Res<(), Packet> {
        if self.position < Self::get_max_size() {
            self.data.insert(self.position, value);
            self.position += 1;
            Ok(())
        } else {
            Err(Packet::Write {
                max_size: Self::get_max_size(),
            })
        }
    }
    pub fn write_bytes(&mut self, value: &[u8]) -> anyhow::Result<(), Packet> {
        for v in value {
            self.write(*v)?;
        }
        Ok(())
    }
    pub fn write_i8_bytes(&mut self, value: &[i8]) -> anyhow::Result<(), Packet> {
        for v in value {
            self.write_i8(*v)?;
        }
        Ok(())
    }
    pub fn write_string(&mut self, value: Option<&str>) -> Res<(), Packet> {
        if let Some(st) = value {
            self.write_bytes(
                &UTF_16LE
                    .encode(st, EncoderTrap::Strict)
                    .map_err(|_| Packet::Encode("UTF_16LE".to_owned()))?,
            )?;
        }
        self.write_i16(0)
    }
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub fn write_sized_string(&mut self, value: Option<&str>) -> Res<(), Packet> {
        if let Some(st) = value {
            self.write_i16((st.len() & 0xff) as i16)?;
            self.write_bytes(
                &UTF_16LE
                    .encode(st, EncoderTrap::Strict)
                    .map_err(|_| Packet::Encode("UTF_16LE".to_owned()))?,
            )?;
        } else {
            self.write_i16(0)?;
        }
        Ok(())
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn write_i8(&mut self, value: i8) -> Res<(), Packet> {
        self.write(value as u8)
    }

    pub fn write_u8(&mut self, value: u8) -> Res<(), Packet> {
        self.write(value)
    }

    pub fn write_bool(&mut self, value: bool) -> Res<(), Packet> {
        self.write_u8(u8::from(value))
    }
    #[allow(clippy::cast_sign_loss)]
    pub fn write_i16(&mut self, value: i16) -> Res<(), Packet> {
        self.write((value & 0xff) as u8)?;
        self.write(((value >> 8) & 0xff) as u8)
    }

    pub fn write_u16(&mut self, value: u16) -> Res<(), Packet> {
        self.write((value & 0xff) as u8)?;
        self.write(((value >> 8) & 0xff) as u8)
    }
    pub fn write_i16_from_bool(&mut self, value: bool) -> Res<(), Packet> {
        self.write_i16(i16::from(value))
    }
    #[allow(clippy::cast_sign_loss)]
    pub fn write_i32(&mut self, value: i32) -> Res<(), Packet> {
        self.write((value & 0xff) as u8)?;
        self.write(((value >> 8) & 0xff) as u8)?;
        self.write(((value >> 16) & 0xff) as u8)?;
        self.write(((value >> 24) & 0xff) as u8)?;
        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> Res<(), Packet> {
        self.write((value & 0xff) as u8)?;
        self.write(((value >> 8) & 0xff) as u8)?;
        self.write(((value >> 16) & 0xff) as u8)?;
        self.write(((value >> 24) & 0xff) as u8)?;
        Ok(())
    }
    pub fn write_i32_from_bool(&mut self, value: bool) -> Res<(), Packet> {
        self.write_i32(i32::from(value))
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn write_i64(&mut self, value: i64) -> Res<(), Packet> {
        self.write((value & 0xff) as u8)?;
        self.write(((value >> 8) & 0xff) as u8)?;
        self.write(((value >> 16) & 0xff) as u8)?;
        self.write(((value >> 24) & 0xff) as u8)?;
        self.write(((value >> 32) & 0xff) as u8)?;
        self.write(((value >> 40) & 0xff) as u8)?;
        self.write(((value >> 48) & 0xff) as u8)?;
        self.write(((value >> 56) & 0xff) as u8)?;
        Ok(())
    }
    pub fn write_i64_from_bool(&mut self, value: bool) -> Res<(), Packet> {
        self.write_i64(i64::from(value))
    }
    #[allow(clippy::cast_possible_wrap)]
    pub fn write_f32(&mut self, value: f32) -> Res<(), Packet> {
        self.write_i32(value.to_bits() as i32)
    }
    #[allow(clippy::cast_possible_wrap)]
    pub fn write_f64(&mut self, value: f64) -> Res<(), Packet> {
        self.write_i64(value.to_bits() as i64)
    }
    #[must_use]
    pub fn get_cursor_position(&self) -> usize {
        self.position
    }
    #[must_use]
    pub fn get_size(&self) -> usize {
        self.position
    }

    pub fn get_data_mut(&mut self) -> &mut [u8] {
        // Add size info at start (unsigned short - max size 65535).
        self.write_packet_size();
        &mut self.data[0..self.position]
    }

    pub fn get_data(&mut self) -> Vec<u8> {
        // Add size info at start (unsigned short - max size 65535).
        self.write_packet_size();
        self.data[0..self.position].to_vec()
    }

    pub fn write_packet_size(&mut self) {
        self.data[0] = (self.position & 0xff) as u8;
        self.data[1] = ((self.position >> 8) & 0xffff) as u8;
    }

    pub fn write_padding(&mut self) -> Res<(), Packet> {
        self.write_i32(0)?;
        let padding = (self.get_size() - 2) % 8;
        if padding != 0 {
            for _ in padding..8 {
                self.write_u8(0)?;
            }
        }
        self.write_packet_size();
        Ok(())
    }

    pub fn resize_buffer(&mut self, size: usize) {
        self.data.reserve(size);
        self.data.resize(size, 0);
    }

    #[must_use]
    pub fn get_max_size() -> usize {
        65535
    }
}
