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
    #[allow(clippy::cast_possible_truncation)]
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), Packet> {
        if !self.is_enabled {
            self.is_enabled = true;
            return Ok(());
        }
        let mut x_or = 0u8;
        let size = data.len();
        for i in 0..size {
            let encrypted = data[i]; // Equivalent to Byte.toUnsignedInt in Java
            data[i] = encrypted ^ self.in_key[i & 15] ^ x_or; // XOR operation and write byte back
            x_or = encrypted;
        }

        // Shift key
        let mut old = u32::from(self.in_key[8]);
        old |= u32::from(self.in_key[9]) << 8;
        old |= u32::from(self.in_key[10]) << 16;
        old |= u32::from(self.in_key[11]) << 24;

        old += size as u32;
        self.in_key[8] = (old & 0xff) as u8;
        self.in_key[9] = ((old >> 8) & 0xff) as u8;
        self.in_key[10] = ((old >> 16) & 0xff) as u8;
        self.in_key[11] = ((old >> 24) & 0xff) as u8;
        Ok(())
    }
    #[allow(clippy::cast_possible_truncation)]
    pub fn encrypt(&mut self, data: &mut [u8]) {
        if !self.is_enabled {
            self.is_enabled = true;
            return;
        }
        let mut encrypted = 0;
        let size = data.len();
        for i in 0..size {
            let raw = u32::from(data[i]);
            encrypted ^= raw ^ u32::from(self.out_key[i & 0x0f]);
            data[i] = encrypted as u8;
        }

        // Shift key
        let mut old = u32::from(self.out_key[8]);
        old |= u32::from(self.out_key[9]) << 8;
        old |= u32::from(self.out_key[10]) << 16;
        old |= u32::from(self.out_key[11]) << 24;
        old += size as u32;

        self.out_key[8] = (old & 0xff) as u8;
        self.out_key[9] = ((old >> 8) & 0xff) as u8;
        self.out_key[10] = ((old >> 16) & 0xff) as u8;
        self.out_key[11] = ((old >> 24) & 0xff) as u8;
    }
}
