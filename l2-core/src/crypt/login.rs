use crate::errors::Packet;
use blowfish::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use blowfish::BlowfishLE;

#[derive(Debug, Clone)]
pub struct Encryption {
    cipher: BlowfishLE,
}

impl Encryption {
    #[must_use]
    pub fn new(key: &[u8]) -> Self {
        Self::from_u8_key(key)
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn from_u8_key(key: &[u8]) -> Self {
        let cipher = BlowfishLE::new_from_slice(key).expect("Received invalid blowfish key");
        Encryption { cipher }
    }

    fn calculate_checksum_block(raw: &[u8]) -> (u32, u32) {
        let mut checksum: u32 = 0;
        let count = raw.len();
        let mut check: u32 = 0;
        let mut offset = 0;
        while offset < count {
            check = u32::from(raw[offset]) & 0xff;
            check |= (u32::from(raw[offset + 1]) << 0x08) & 0xff00;
            check |= (u32::from(raw[offset + 2]) << 0x10) & 0x00ff_0000;
            check |= (u32::from(raw[offset + 3]) << 0x18) & 0xff00_0000;
            offset += 4;
            if offset < count {
                checksum ^= check;
            }
        }
        (check, checksum)
    }

    #[must_use]
    pub fn verify_checksum(raw: &[u8]) -> bool {
        let size = raw.len();
        if size & 3 != 0 || size <= 4 {
            return false;
        }
        let (check, checksum) = Self::calculate_checksum_block(raw);
        check == checksum
    }

    pub fn append_checksum(raw: &mut [u8]) {
        let (_, checksum) = Self::calculate_checksum_block(raw);
        let last = raw.len() - 4; //modify last 4 bytes, starting from offset
        raw[last] = (checksum & 0xff) as u8;
        raw[last + 1] = ((checksum >> 0x08) & 0xff) as u8;
        raw[last + 2] = ((checksum >> 0x10) & 0xff) as u8;
        raw[last + 3] = ((checksum >> 0x18) & 0xff) as u8;
    }
    #[allow(clippy::similar_names)]
    pub fn enc_xor_pass(raw: &mut [u8], offset: usize, size: usize, key: u32) {
        let stop = size - 8;
        let mut pos = 4 + offset;
        let mut ecx = key; // Initial xor key

        while pos < stop {
            let edx = u32::from(raw[pos]) & 0xFF
                | (u32::from(raw[pos + 1]) & 0xFF) << 8
                | (u32::from(raw[pos + 2]) & 0xFF) << 16
                | (u32::from(raw[pos + 3]) & 0xFF) << 24;

            ecx = ecx.wrapping_add(edx);
            let edx_xor = edx ^ ecx;

            raw[pos] = (edx_xor & 0xFF) as u8;
            raw[pos + 1] = ((edx_xor >> 8) & 0xFF) as u8;
            raw[pos + 2] = ((edx_xor >> 16) & 0xFF) as u8;
            raw[pos + 3] = ((edx_xor >> 24) & 0xFF) as u8;

            pos += 4;
        }

        raw[pos] = (ecx & 0xFF) as u8;
        raw[pos + 1] = ((ecx >> 8) & 0xFF) as u8;
        raw[pos + 2] = ((ecx >> 16) & 0xFF) as u8;
        raw[pos + 3] = ((ecx >> 24) & 0xFF) as u8;
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn decrypt(&self, raw: &mut [u8]) -> Result<(), Packet> {
        let size = raw.len();
        let offset = 0;
        if size % 8 > 0 || offset + size > raw.len() {
            return Err(Packet::DecryptBlowfishError);
        }
        for chunk in raw.chunks_mut(8) {
            self.cipher.decrypt_block(chunk.into());
        }
        Ok(())
    }
    pub fn encrypt(&self, raw: &mut [u8]) {
        for chunk in raw.chunks_mut(8) {
            self.cipher.encrypt_block(chunk.into());
        }
    }
}

#[cfg(test)]
mod test {
    use crate::crypt::login::Encryption;

    #[test]
    fn test_decrypt_auth_gg() {
        let mut data = [
            123, 127, 180, 251, 24, 25, 187, 239, 212, 216, 154, 37, 43, 197, 224, 71, 212, 216,
            154, 37, 43, 197, 224, 71, 207, 203, 144, 187, 61, 185, 36, 74, 212, 216, 154, 37, 43,
            197, 224, 71,
        ];
        let key = [
            62, 246, 48, 159, 201, 154, 143, 146, 235, 160, 96, 51, 237, 105, 97, 227,
        ];
        let decrypted = [
            7, 219, 128, 157, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 219,
            128, 157, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let decryptor = Encryption::from_u8_key(&key);
        let res = decryptor.decrypt(&mut data);
        assert!(res.is_ok(), "Result must be ok");
        assert_eq!(
            data, decrypted,
            "Auth gg packet should be decrypted correctly"
        );
    }

    #[test]
    fn test_decrypt_auth_gg_2() {
        let mut data = [
            77, 76, 88, 220, 187, 217, 93, 78, 12, 173, 190, 109, 220, 201, 213, 212, 12, 173, 190,
            109, 220, 201, 213, 212, 203, 11, 135, 129, 151, 169, 216, 152, 12, 173, 190, 109, 220,
            201, 213, 212,
        ];
        let decrypted = [
            7, 30, 251, 17, 214, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 209, 30,
            251, 17, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let key = [
            12, 84, 204, 79, 78, 136, 249, 67, 63, 70, 44, 61, 28, 224, 9, 31,
        ];
        let decryptor = Encryption::from_u8_key(&key);
        let res = decryptor.decrypt(&mut data);
        assert!(res.is_ok(), "Result must be ok");
        assert_eq!(
            data, decrypted,
            "Auth gg packet should be decrypted correctly"
        );
    }

    #[test]
    fn test_append_checksum() {
        let mut data: Vec<u8> = vec![
            146, 0, 0, 6, 1, 0, 0, 129, 0, 0, 0, 0, 128, 29, 57, 69, 167, 240, 165, 98, 143, 2,
            173, 193, 96, 200, 229, 53, 241, 30, 179, 153, 133, 105, 24, 207, 36, 201, 102, 38,
            231, 177, 44, 77, 156, 76, 85, 99, 160, 111, 109, 104, 151, 52, 255, 76, 13, 130, 181,
            185, 211, 74, 209, 215, 161, 14, 17, 203, 141, 145, 114, 132, 18, 185, 219, 116, 81,
            14, 140, 64, 54, 154, 48, 160, 25, 213, 189, 75, 197, 103, 222, 53, 105, 218, 123, 75,
            103, 208, 251, 47, 44, 21, 14, 178, 62, 117, 188, 171, 87, 32, 211, 79, 42, 27, 152,
            96, 66, 111, 110, 238, 133, 142, 127, 76, 38, 85, 38, 27, 252, 58, 165, 51, 164, 88,
            87, 93, 205, 91, 240, 249, 0, 0, 0, 0, 0, 0,
        ];
        let data_expected = expected_checksum();
        let size = data.len();
        Encryption::append_checksum(&mut data[2..size]);
        assert_eq!(data, data_expected, "Append checksum must work");
    }

    #[test]
    fn test_verify_checksum() {
        let data = expected_checksum();
        let result = Encryption::verify_checksum(&data[2..data.len()]);
        assert!(result, "Verify checksum must work");
    }

    #[test]
    fn verify_checksum_2() {
        let data = [
            1, 1, 0, 0, 97, 30, 208, 7, 0, 0, 16, 0, 0, 0, 213, 41, 148, 192, 183, 195, 221, 65,
            246, 143, 230, 10, 163, 117, 66, 16, 5, 0, 0, 0, 49, 0, 48, 0, 46, 0, 51, 0, 55, 0, 46,
            0, 49, 0, 50, 0, 57, 0, 46, 0, 48, 0, 47, 0, 50, 0, 52, 0, 0, 0, 49, 0, 48, 0, 46, 0,
            51, 0, 55, 0, 46, 0, 49, 0, 50, 0, 57, 0, 46, 0, 50, 0, 0, 0, 49, 0, 48, 0, 46, 0, 50,
            0, 49, 0, 49, 0, 46, 0, 53, 0, 53, 0, 46, 0, 48, 0, 47, 0, 50, 0, 52, 0, 0, 0, 49, 0,
            48, 0, 46, 0, 50, 0, 49, 0, 49, 0, 46, 0, 53, 0, 53, 0, 46, 0, 50, 0, 0, 0, 49, 0, 57,
            0, 50, 0, 46, 0, 49, 0, 54, 0, 56, 0, 46, 0, 50, 0, 48, 0, 46, 0, 48, 0, 47, 0, 50, 0,
            52, 0, 0, 0, 49, 0, 57, 0, 50, 0, 46, 0, 49, 0, 54, 0, 56, 0, 46, 0, 50, 0, 48, 0, 46,
            0, 49, 0, 48, 0, 50, 0, 0, 0, 49, 0, 50, 0, 55, 0, 46, 0, 48, 0, 46, 0, 48, 0, 46, 0,
            48, 0, 47, 0, 56, 0, 0, 0, 49, 0, 50, 0, 55, 0, 46, 0, 48, 0, 46, 0, 48, 0, 46, 0, 49,
            0, 0, 0, 48, 0, 46, 0, 48, 0, 46, 0, 48, 0, 46, 0, 48, 0, 47, 0, 48, 0, 0, 0, 56, 0,
            57, 0, 46, 0, 50, 0, 51, 0, 52, 0, 46, 0, 50, 0, 52, 0, 53, 0, 46, 0, 49, 0, 54, 0, 0,
            0, 0, 0, 0, 0, 133, 132, 217, 23,
        ];
        let result = Encryption::verify_checksum(&data);
        assert!(result, "Verify checksum must work");
    }

    #[test]
    fn test_decrypt_auth_login() {
        let mut data = [
            137, 139, 65, 222, 56, 149, 202, 165, 122, 16, 248, 96, 30, 90, 217, 42, 255, 206, 151,
            22, 238, 153, 189, 236, 206, 179, 95, 83, 48, 120, 66, 158, 220, 186, 172, 73, 190,
            249, 49, 87, 41, 167, 163, 23, 78, 146, 252, 51, 192, 252, 122, 110, 35, 79, 39, 129,
            128, 60, 191, 153, 41, 238, 195, 77, 15, 87, 71, 165, 33, 146, 137, 125, 239, 119, 42,
            140, 118, 110, 77, 121, 146, 111, 191, 173, 196, 11, 22, 220, 164, 237, 18, 124, 110,
            70, 53, 168, 168, 209, 252, 199, 6, 202, 62, 145, 28, 215, 190, 52, 45, 58, 125, 2, 4,
            47, 8, 78, 92, 195, 78, 238, 138, 249, 115, 133, 218, 240, 67, 68, 71, 148, 152, 222,
            160, 225, 146, 119, 31, 222, 176, 6, 206, 222, 160, 207, 91, 194, 138, 75, 137, 0, 84,
            43, 110, 80, 78, 102, 109, 58, 227, 160, 186, 19, 98, 121, 140, 244, 235, 96, 105, 196,
            125, 18, 48, 188, 145, 107, 205, 127, 114, 213, 189, 104, 248, 179, 59, 110, 87, 226,
            37, 189, 15, 239, 31, 202, 20, 105, 51, 143, 139, 102, 76, 19, 166, 26, 86, 255, 220,
            197, 6, 111, 116, 198, 231, 92, 249, 253, 123, 247, 90, 69, 140, 145, 43, 84, 156, 144,
            16, 133, 177, 18, 85, 235, 234, 172, 169, 76, 228, 231, 80, 196, 114, 151, 46, 130, 89,
            135, 162, 245, 40, 139, 200, 100, 200, 183, 216, 32, 88, 174, 255, 24, 8, 117, 225,
            209, 7, 244, 251, 149, 192, 150, 213, 254, 104, 251, 92, 86, 64, 15, 226, 157, 16, 190,
            100, 10, 69, 45, 183, 82, 171, 96, 192, 121, 153, 111, 24, 140, 74, 168, 45, 83, 231,
            7, 209, 67, 191, 2, 56, 110, 216, 208, 149, 72, 7, 244, 251, 149, 192, 150, 213, 254,
        ];
        let key = [
            61, 103, 234, 138, 3, 212, 186, 146, 11, 149, 0, 133, 31, 49, 119, 198,
        ];
        let decrypted = [
            0, 80, 79, 135, 38, 17, 91, 97, 10, 48, 225, 116, 84, 137, 249, 99, 172, 52, 157, 202,
            67, 62, 189, 136, 165, 137, 162, 199, 119, 49, 39, 76, 16, 53, 113, 196, 180, 131, 254,
            130, 77, 237, 122, 38, 153, 102, 152, 71, 175, 209, 221, 75, 32, 4, 163, 170, 189, 109,
            218, 233, 116, 187, 24, 213, 5, 106, 174, 7, 112, 253, 122, 203, 222, 196, 122, 64,
            145, 164, 156, 247, 92, 150, 242, 255, 46, 133, 235, 32, 176, 186, 84, 250, 33, 46,
            166, 166, 207, 219, 213, 179, 189, 100, 219, 218, 133, 103, 220, 200, 111, 27, 116,
            236, 73, 196, 99, 109, 12, 149, 103, 81, 60, 36, 202, 171, 58, 229, 97, 58, 169, 113,
            76, 252, 102, 219, 50, 173, 237, 57, 176, 9, 166, 210, 149, 4, 98, 118, 89, 153, 92,
            86, 84, 79, 176, 176, 68, 221, 146, 100, 69, 126, 207, 51, 27, 104, 211, 56, 230, 209,
            218, 76, 51, 119, 108, 80, 70, 53, 121, 134, 95, 148, 182, 135, 182, 20, 245, 120, 76,
            135, 119, 235, 13, 164, 225, 201, 213, 8, 238, 191, 230, 49, 5, 125, 250, 159, 197,
            214, 232, 222, 208, 195, 213, 4, 183, 210, 85, 169, 181, 29, 190, 106, 189, 252, 164,
            112, 222, 184, 18, 223, 10, 156, 64, 182, 95, 150, 6, 231, 76, 65, 81, 195, 254, 16,
            201, 221, 11, 13, 115, 131, 108, 160, 196, 156, 199, 116, 45, 159, 132, 131, 94, 215,
            128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 201, 60, 201,
            172, 185, 36, 197, 189, 152, 64, 89, 234, 166, 34, 61, 246, 0, 0, 0, 0, 125, 129, 4,
            174, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let decryptor = Encryption::from_u8_key(&key);
        let res = decryptor.decrypt(&mut data);
        assert!(res.is_ok(), "Result must be ok");
        assert_eq!(
            data, decrypted,
            "Auth gg packet should be decrypted correctly"
        );
    }
    fn expected_checksum() -> [u8; 146] {
        [
            146, 0, 0, 6, 1, 0, 0, 129, 0, 0, 0, 0, 128, 29, 57, 69, 167, 240, 165, 98, 143, 2,
            173, 193, 96, 200, 229, 53, 241, 30, 179, 153, 133, 105, 24, 207, 36, 201, 102, 38,
            231, 177, 44, 77, 156, 76, 85, 99, 160, 111, 109, 104, 151, 52, 255, 76, 13, 130, 181,
            185, 211, 74, 209, 215, 161, 14, 17, 203, 141, 145, 114, 132, 18, 185, 219, 116, 81,
            14, 140, 64, 54, 154, 48, 160, 25, 213, 189, 75, 197, 103, 222, 53, 105, 218, 123, 75,
            103, 208, 251, 47, 44, 21, 14, 178, 62, 117, 188, 171, 87, 32, 211, 79, 42, 27, 152,
            96, 66, 111, 110, 238, 133, 142, 127, 76, 38, 85, 38, 27, 252, 58, 165, 51, 164, 88,
            87, 93, 205, 91, 240, 249, 0, 0, 189, 153, 155, 43,
        ]
    }
}
