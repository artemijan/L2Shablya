use anyhow::{anyhow, bail, Result};
use encoding::all::UTF_16LE;
use encoding::{DecoderTrap, Encoding};

#[derive(Debug, Clone)]
pub struct ReadablePacketBuffer<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ReadablePacketBuffer<'a> {
    #[must_use]
    pub fn new(bytes: &'a [u8]) -> Self {
        ReadablePacketBuffer { bytes, position: 0 }
    }

    // Helper method to check buffer bounds and advance position
    fn check_and_advance(&mut self, length: usize) -> Result<()> {
        if self.position + length > self.bytes.len() {
            bail!(
                "Not enough bytes available. Requested: {}, Remaining: {}",
                length,
                self.bytes.len() - self.position
            );
        }
        self.position += length;
        Ok(())
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_boolean(&mut self) -> Result<bool> {
        Ok(self.read_byte()? != 0)
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_c_utf16le_string(&mut self) -> Result<String> {
        let mut result = String::new();
        let bytes = &self.bytes[self.position..];
        let mut i = 0;
        while i + 1 < bytes.len() {
            let utf16_value = u16::from_le_bytes([bytes[i], bytes[i + 1]]);
            i += 2;
            if utf16_value == 0 {
                break;
            }
            if let Some(ch) = char::from_u32(u32::from(utf16_value)) {
                result.push(ch);
            } else {
                result.push('\u{FFFD}'); // Unicode replacement character
            }
        }
        self.position += i;
        Ok(result)
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_n_strings(&mut self, count: usize) -> Result<Vec<String>> {
        (0..count).map(|_| self.read_c_utf16le_string()).collect()
    }

    /// # Errors
    /// - it's not enough bytes
    #[allow(clippy::cast_sign_loss)]
    pub fn read_sized_string(&mut self) -> Result<String> {
        let length = self.read_i16()? as usize;
        let bytes = self.read_bytes(length * 2)?;
        UTF_16LE
            .decode(bytes, DecoderTrap::Replace)
            .map_err(|e| anyhow!("Can not decode string from bytes {e}"))
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_bytes(&mut self, length: usize) -> Result<&[u8]> {
        self.check_and_advance(length)?;
        Ok(&self.bytes[self.position - length..self.position])
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_byte(&mut self) -> Result<u8> {
        let byte = self
            .bytes
            .get(self.position)
            .copied()
            .ok_or_else(|| anyhow!("Buffer underflow"))?;
        self.position += 1;
        Ok(byte)
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_i16(&mut self) -> Result<i16> {
        self.check_and_advance(2)?;
        let chunk = &self.bytes[self.position - 2..self.position]; // Slice access
        Ok(i16::from_le_bytes(
            chunk
                .try_into()
                .map_err(|_| anyhow!("Invalid i16 byte slice"))?,
        ))
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_u16(&mut self) -> Result<u16> {
        self.check_and_advance(2)?;
        let chunk = &self.bytes[self.position - 2..self.position]; // Slice access
        Ok(u16::from_le_bytes(
            chunk
                .try_into()
                .map_err(|_| anyhow!("Invalid u16 byte slice"))?,
        ))
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_i32(&mut self) -> Result<i32> {
        self.check_and_advance(4)?;
        let chunk = &self.bytes[self.position - 4..self.position]; // Slice access
        Ok(i32::from_le_bytes(
            chunk
                .try_into()
                .map_err(|_| anyhow!("Invalid i32 byte slice"))?,
        ))
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_u32(&mut self) -> Result<u32> {
        self.check_and_advance(4)?;
        let chunk = &self.bytes[self.position - 4..self.position]; // Slice access
        Ok(u32::from_le_bytes(
            chunk
                .try_into()
                .map_err(|_| anyhow!("Invalid u32 byte slice"))?,
        ))
    }

    /// # Errors
    /// - it's not enough bytes
    pub fn read_i64(&mut self) -> Result<i64> {
        self.check_and_advance(8)?;
        let chunk = &self.bytes[self.position - 8..self.position]; // Slice access
        Ok(i64::from_le_bytes(
            chunk
                .try_into()
                .map_err(|_| anyhow!("Invalid i64 byte slice"))?,
        ))
    }

    /// # Errors
    /// - it's not enough bytes
    #[allow(clippy::cast_sign_loss)]
    pub fn read_float(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.read_i32()? as u32))
    }

    /// # Errors
    /// - it's not enough bytes
    #[allow(clippy::cast_sign_loss)]
    pub fn read_double(&mut self) -> Result<f64> {
        Ok(f64::from_bits(self.read_i64()? as u64))
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

    #[test]
    fn test_read_bool_false() {
        let arr = &[0];
        let mut buff = ReadablePacketBuffer::new(arr);
        let value = buff.read_boolean().unwrap();
        assert!(!value);
    }

    #[test]
    fn test_read_bool_true() {
        let arr = &[1];
        let mut buff = ReadablePacketBuffer::new(arr);
        let value = buff.read_boolean().unwrap();
        assert!(value);
    }

    // Add more tests for other methods...
}
