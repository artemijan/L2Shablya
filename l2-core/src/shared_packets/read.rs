use encoding::all::UTF_16LE;
use encoding::{DecoderTrap, Encoding};

#[derive(Debug, Clone)]
pub struct ReadablePacketBuffer {
    bytes: Vec<u8>,
    position: usize,
}

#[allow(unused)]
impl ReadablePacketBuffer {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        ReadablePacketBuffer { bytes, position: 0 }
    }

    pub fn read_boolean(&mut self) -> bool {
        self.read_byte() != 0
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn read_string(&mut self) -> String {
        let mut result = String::new();
        loop {
            let char_id = self.read_i16() as u8;
            if char_id == 0 {
                break;
            }
            result.push(char_id as char);
        }
        result
    }

    pub fn read_n_strings(&mut self, count: usize) -> Vec<String> {
        let mut hosts = Vec::with_capacity(count);
        for _ in 0..count {
            let s = &self.read_string();
            hosts.push(s.clone());
        }
        hosts
    }

    #[allow(clippy::cast_sign_loss, clippy::missing_panics_doc)]
    pub fn read_sized_string(&mut self) -> String {
        let length = self.read_i16() as usize;
        let bytes = self.read_bytes(length * 2);
        // safe to use unwrap, because decoder is "Replace", error here unreachable
        UTF_16LE
            .decode(&bytes, DecoderTrap::Replace)
            .expect("Can not decode string from bytes")
    }
    pub fn read_bytes(&mut self, length: usize) -> Vec<u8> {
        let result = &self.bytes[self.position..self.position + length];
        self.position += length;
        result.to_vec()
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.bytes[self.position];
        self.position += 1;
        byte
    }

    pub fn read_i16(&mut self) -> i16 {
        let short = i16::from_le_bytes([self.bytes[self.position], self.bytes[self.position + 1]]);
        self.position += 2;
        short
    }

    pub fn read_u16(&mut self) -> u16 {
        let short = u16::from_le_bytes([self.bytes[self.position], self.bytes[self.position + 1]]);
        self.position += 2;
        short
    }

    pub fn read_i32(&mut self) -> i32 {
        let int = i32::from_le_bytes([
            self.bytes[self.position],
            self.bytes[self.position + 1],
            self.bytes[self.position + 2],
            self.bytes[self.position + 3],
        ]);
        self.position += 4;
        int
    }

    pub fn read_u32(&mut self) -> u32 {
        let int = u32::from_le_bytes([
            self.bytes[self.position],
            self.bytes[self.position + 1],
            self.bytes[self.position + 2],
            self.bytes[self.position + 3],
        ]);
        self.position += 4;
        int
    }

    pub fn read_i64(&mut self) -> i64 {
        let long = i64::from_le_bytes([
            self.bytes[self.position],
            self.bytes[self.position + 1],
            self.bytes[self.position + 2],
            self.bytes[self.position + 3],
            self.bytes[self.position + 4],
            self.bytes[self.position + 5],
            self.bytes[self.position + 6],
            self.bytes[self.position + 7],
        ]);
        self.position += 8;
        long
    }
    #[allow(clippy::cast_sign_loss)]
    pub fn read_float(&mut self) -> f32 {
        f32::from_bits(self.read_i32() as u32)
    }
    #[allow(clippy::cast_sign_loss)]
    pub fn read_double(&mut self) -> f64 {
        f64::from_bits(self.read_i64() as u64)
    }

    #[must_use]
    pub fn get_remaining_length(&self) -> usize {
        self.bytes.len() - self.position
    }

    #[must_use]
    pub fn get_length(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::Packet;
    use encoding::all::UTF_16LE;
    use encoding::{EncoderTrap, Encoding};

    #[test]
    fn test_read_bool_false() {
        let mut buff = ReadablePacketBuffer::new(vec![0]);
        let value = buff.read_boolean();
        assert!(!value, "Should be false");
    }

    #[test]
    fn test_read_bool_true() {
        let mut buff = ReadablePacketBuffer::new(vec![1]);
        let value = buff.read_boolean();
        assert!(value, "Should be true");
    }

    #[test]
    fn test_read_string() {
        let bytes = encode_str("test me");
        let mut buff = ReadablePacketBuffer::new(bytes);
        let value = buff.read_string();
        assert_eq!(value, "test me");
    }

    #[test]
    fn test_read_n_strings() {
        let expected_data = vec!["test", " ", "me", " ", "please"];
        let data_len = expected_data.len();
        let mut bytes = vec![];
        for v in &expected_data {
            bytes.extend(encode_str(v));
        }
        let mut buff = ReadablePacketBuffer::new(bytes);
        let value = buff.read_n_strings(data_len);
        assert_eq!(value, expected_data);
    }

    #[test]
    fn test_read_sized_string() {
        let mut bytes: Vec<u8> = vec![7, 0]; // length of the string is 7 chars in Little Endian
        bytes.extend(encode_str("test me"));
        let mut buff = ReadablePacketBuffer::new(bytes);
        let value = buff.read_sized_string();
        assert_eq!(value, "test me");
    }

    #[test]
    fn test_read_bytes() {
        let bytes: Vec<u8> = vec![7, 0, 89, 76, 65, 78, 98];
        let mut buff = ReadablePacketBuffer::new(bytes);
        let value = buff.read_bytes(3);
        assert_eq!(value, vec![7, 0, 89]);
    }

    #[test]
    fn test_read_byte() {
        let bytes: Vec<u8> = vec![9, 0, 89, 76];
        let mut buff = ReadablePacketBuffer::new(bytes);
        let value = buff.read_byte();
        assert_eq!(value, 9);
    }
    #[test]
    fn test_read_i16() {
        let bytes: Vec<u8> = vec![11, 0, 89, 76];
        let mut buff = ReadablePacketBuffer::new(bytes);
        let value = buff.read_i16();
        assert_eq!(value, 11);
    }

    #[test]
    fn test_read_i32() {
        let val: i32 = 789_876;
        let bytes = val.to_le_bytes();
        let mut buff = ReadablePacketBuffer::new(bytes.to_vec());
        let value = buff.read_i32();
        assert_eq!(value, val);
    }

    #[test]
    fn test_read_i64() {
        let val: i64 = 78_987_678;
        let bytes = val.to_le_bytes();
        let mut buff = ReadablePacketBuffer::new(bytes.to_vec());
        let value = buff.read_i64();
        assert_eq!(value, val);
    }

    fn encode_str(s: &str) -> Vec<u8> {
        let mut bytes = UTF_16LE
            .encode(s, EncoderTrap::Strict)
            .map_err(|_| Packet::Encode("UTF_16LE".to_owned()))
            .unwrap();
        bytes.extend(vec![0, 0]);
        bytes
    }
}
