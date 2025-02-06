use crate::crypt::BLOWFISH_KEY_SIZE;
use crate::errors::Packet;
use anyhow::bail;

#[derive(Debug)]
pub struct GameClientEncryption {
    in_key: [u8; BLOWFISH_KEY_SIZE],
    out_key: [u8; BLOWFISH_KEY_SIZE],
    is_enabled: bool,
}
impl GameClientEncryption {
    /// # Errors
    /// - when the key is not equal to 16 bytes
    pub fn new(key: &[u8]) -> anyhow::Result<GameClientEncryption> {
        if key.len() != BLOWFISH_KEY_SIZE {
            bail!(Packet::SendPacketError);
        }
        let mut key_array: [u8; 16] = [0; 16];
        key_array.clone_from_slice(&key[..16]);
        //because of implicit copy, these will be two different arrays
        Ok(Self {
            in_key: key_array,  //copy 1
            out_key: key_array, //copy 2
            is_enabled: false,
        })
    }

    /// # Errors
    /// - when data size is too big, in this case data.len will try to convert to i32 and fail
    pub fn decrypt(&mut self, data: &mut [u8]) -> anyhow::Result<()> {
        if !self.is_enabled {
            self.is_enabled = true;
            return Ok(());
        }

        let mut x_or = 0u8;
        for (i, byte) in data.iter_mut().enumerate() {
            let encrypted = *byte;
            *byte ^= self.in_key[i & 15] ^ x_or;
            x_or = encrypted;
        }
        // Shift key efficiently
        let old = u32::from_le_bytes(self.in_key[8..12].try_into()?) + u32::try_from(data.len())?;
        self.in_key[8..12].copy_from_slice(&old.to_le_bytes());
        Ok(())
    }
    /// # Errors
    /// - when data size is too big, in this case data.len will try to convert to i32 and fail
    pub fn encrypt(&mut self, data: &mut [u8]) -> anyhow::Result<()> {
        if !self.is_enabled {
            self.is_enabled = true;
            return Ok(());
        }

        let mut encrypted = 0u8;
        for (i, byte) in data.iter_mut().enumerate() {
            encrypted ^= *byte ^ self.out_key[i & 15];
            *byte = encrypted;
        }

        // Shift key efficiently
        let old = u32::from_le_bytes(self.out_key[8..12].try_into()?) + u32::try_from(data.len())?;
        self.out_key[8..12].copy_from_slice(&old.to_le_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::crypt::game::GameClientEncryption;

    #[test]
    fn encrypt_works() {
        let the_key = [23, 17, 0, 1, 78, 198, 12, 8, 56, 110, 10, 9, 121, 34, 6, 8];
        let mut encryption = GameClientEncryption::new(&the_key).unwrap();
        let mut data = [0, 12, 34, 56, 78, 198, 12, 32, 56, 0, 1, 9, 12, 34, 56];
        let unchanged = data; //copy
        encryption.encrypt(&mut data).unwrap();
        assert_eq!(data, unchanged);
        encryption.encrypt(&mut data).unwrap();
        assert_ne!(unchanged, data);
        assert_eq!(
            data,
            [23, 10, 40, 17, 17, 17, 17, 57, 57, 87, 92, 92, 41, 41, 23]
        );
        encryption.encrypt(&mut data).unwrap();
        assert_eq!(
            data,
            [0, 27, 51, 35, 124, 171, 182, 135, 249, 192, 150, 195, 147, 152, 137]
        );
    }
    #[test]
    fn decrypt_works() {
        let the_key = [23, 17, 0, 1, 78, 198, 12, 8, 56, 110, 10, 9, 121, 34, 6, 8];
        let mut encryption = GameClientEncryption::new(&the_key).unwrap();
        let mut data = [0, 12, 34, 56, 78, 198, 12, 32, 56, 0, 1, 9, 12, 34, 56];
        let unchanged = data; //copy
        encryption.decrypt(&mut data).unwrap();
        assert_eq!(data, unchanged);
        encryption.decrypt(&mut data).unwrap();
        assert_ne!(unchanged, data);
        assert_eq!(
            data,
            [23, 29, 46, 27, 56, 78, 198, 36, 32, 86, 11, 1, 124, 12, 28]
        );
        encryption.decrypt(&mut data).unwrap();
        assert_eq!(
            data,
            [0, 27, 51, 52, 109, 176, 132, 234, 67, 24, 87, 3, 4, 82, 22]
        );
    }
}
