use crate::common::errors::PacketErrors;
use anyhow::Result as Res;
use encoding::all::UTF_16LE;
use encoding::{EncoderTrap, Encoding};

#[derive(Debug, Clone)]
pub struct SendablePacketBuffer {
    data: Vec<u8>,
    position: usize,
}

#[allow(unused)]
impl SendablePacketBuffer {
    pub fn new() -> Self {
        SendablePacketBuffer {
            data: vec![0; 32],
            position: 2,
        }
    }
    pub fn from_bytes(data: &[u8]) -> Self {
        SendablePacketBuffer {
            data: data.to_vec(),
            position: data.len(),
        }
    }
    pub fn write(&mut self, value: u8) -> Res<(), PacketErrors> {
        if self.position < self.get_max_size() {
            self.data.insert(self.position, value);
            self.position += 1;
            Ok(())
        } else {
            Err(PacketErrors::PacketWrite {
                max_size: self.get_max_size(),
            })
        }
    }
    pub fn write_bytes(&mut self, value: Vec<u8>) -> anyhow::Result<(), PacketErrors> {
        for v in value.iter() {
            self.write(*v)?;
        }
        Ok(())
    }
    pub fn write_i8_bytes(&mut self, value: Vec<i8>) -> anyhow::Result<(), PacketErrors> {
        for v in value.iter() {
            self.write_i8(*v)?;
        }
        Ok(())
    }
    pub fn write_string(&mut self, value: Option<&str>) -> Res<(), PacketErrors> {
        if let Some(st) = value {
            self.write_bytes(
                UTF_16LE
                    .encode(st, EncoderTrap::Strict)
                    .map_err(|_| PacketErrors::Encode("UTF_16LE".to_owned()))?,
            )?;
        }
        self.write_i16(0)
    }
    pub fn write_sized_string(&mut self, value: Option<&str>) -> Res<(), PacketErrors> {
        if let Some(st) = value {
            self.write_i16((st.len() & 0xff) as i16)?;
            self.write_bytes(
                UTF_16LE
                    .encode(st, EncoderTrap::Strict)
                    .map_err(|_| PacketErrors::Encode("UTF_16LE".to_owned()))?,
            )?;
        } else {
            self.write_i16(0)?;
        }
        Ok(())
    }
    pub fn write_i8(&mut self, value: i8) -> Res<(), PacketErrors> {
        self.write(value as u8)
    }

    pub fn write_u8(&mut self, value: u8) -> Res<(), PacketErrors> {
        self.write(value)
    }
    pub fn write_i8_from_bool(&mut self, value: bool) -> Res<(), PacketErrors> {
        self.write_i8(if value { 1 } else { 0 })
    }
    pub fn write_i16(&mut self, value: i16) -> Res<(), PacketErrors> {
        self.write((value & 0xff) as u8)?;
        self.write(((value >> 8) & 0xff) as u8)
    }
    pub fn write_i16_from_bool(&mut self, value: bool) -> Res<(), PacketErrors> {
        self.write_i16(if value { 1 } else { 0 })
    }
    pub fn write_i32(&mut self, value: i32) -> Res<(), PacketErrors> {
        self.write((value & 0xff) as u8)?;
        self.write(((value >> 8) & 0xff) as u8)?;
        self.write(((value >> 16) & 0xff) as u8)?;
        self.write(((value >> 24) & 0xff) as u8)?;
        Ok(())
    }
    pub fn write_i32_from_bool(&mut self, value: bool) -> Res<(), PacketErrors> {
        self.write_i32(if value { 1 } else { 0 })
    }
    pub fn write_i64(&mut self, value: i64) -> Res<(), PacketErrors> {
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
    pub fn write_i64_from_bool(&mut self, value: bool) -> Res<(), PacketErrors> {
        self.write_i64(if value { 1 } else { 0 })
    }
    pub fn write_f32(&mut self, value: f32) -> Res<(), PacketErrors> {
        self.write_i32(value.to_bits() as i32)
    }
    pub fn write_f64(&mut self, value: f64) -> Res<(), PacketErrors> {
        self.write_i64(value.to_bits() as i64)
    }

    pub fn get_cursor_position(&self) -> usize {
        self.position
    }
    pub fn get_size(&self) -> usize {
        self.data.len()
    }
    pub fn get_data(&self) -> Vec<u8> {
        let mut data = self.data[0..self.position].to_vec();
        // Add size info at start (unsigned short - max size 65535).
        data[0] = (self.position & 0xff) as u8;
        data[1] = ((self.position >> 8) & 0xffff) as u8;
        data
    }

    pub fn resize_buffer(&mut self, size: usize) {
        self.data.reserve(size);
        self.data.resize(size, 0);
    }

    pub fn get_max_size(&self) -> usize {
        65535
    }
}
