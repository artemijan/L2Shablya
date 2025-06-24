use anyhow::{bail, Context};
use bytes::{Bytes, BytesMut};
use num_traits::{FromPrimitive, Zero};
use pem::parse;
use rsa;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::{PrivateKeyParts, PublicKeyParts};

#[derive(Debug, Clone)]
pub struct ScrambledRSAKeyPair {
    pair: (rsa::RsaPrivateKey, rsa::RsaPublicKey),
    scrambled_modulus: Vec<u8>,
    modulus: Vec<u8>,
}

pub struct RSAPublicKey {
    key: rsa::RsaPublicKey,
}

impl RSAPublicKey {
    #[must_use]
    pub fn new(key: rsa::RsaPublicKey) -> Self {
        Self { key }
    }
    #[allow(clippy::missing_errors_doc)]
    pub fn from_modulus(modulus: &[u8]) -> anyhow::Result<RSAPublicKey> {
        let modulus = rsa::BigUint::from_bytes_be(modulus);
        // Use the standard public exponent: 65537 (0x10001)
        let exponent = rsa::BigUint::from_u32(65537)
            .context("Failed to create BigUint for public exponent")?;

        // Construct the RSA public key
        let rsa =
            rsa::RsaPublicKey::new(modulus, exponent).context("Failed to create RSA public key")?;
        Ok(RSAPublicKey { key: rsa })
    }

    ///Encryption with No Padding scheme
    #[must_use]
    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let m = rsa::BigUint::from_bytes_be(data);
        m.modpow(self.key.e(), self.key.n()).to_bytes_be()
    }
}

impl ScrambledRSAKeyPair {
    #[must_use]
    pub fn new(key: (rsa::RsaPrivateKey, rsa::RsaPublicKey)) -> Self {
        let (prv, pbc) = key;
        let modulus = pbc.n();
        let mut modulus_bytes = modulus.to_bytes_be();
        //we have to handle the sign -/+ manually
        if modulus < &rsa::BigUint::zero() {
            modulus_bytes.insert(0, 0xFF);
        } else {
            modulus_bytes.insert(0, 0x00);
        }
        ScrambledRSAKeyPair {
            pair: (prv, pbc),
            scrambled_modulus: ScrambledRSAKeyPair::scramble_modulus(modulus_bytes.clone()),
            modulus: modulus_bytes,
        }
    }
    #[must_use]
    pub fn get_scrambled_modulus(&self) -> Vec<u8> {
        self.scrambled_modulus.clone()
    }
    #[must_use]
    pub fn get_modulus(&self) -> Vec<u8> {
        self.modulus.clone()
    }
    #[must_use]
    pub fn scramble_modulus(modulus_bytes: Vec<u8>) -> Vec<u8> {
        let mut scrambled_mod: Vec<u8> = modulus_bytes;

        if scrambled_mod.len() == 0x81 && scrambled_mod[0] == 0x00 {
            let temp: Vec<u8> = scrambled_mod[1..].to_vec();
            scrambled_mod = temp;
        }

        // Step 1: Swap bytes at positions 0x00-0x04 and 0x4d-0x50
        for i in 0..4 {
            (scrambled_mod[i], scrambled_mod[0x4d + i]) =
                (scrambled_mod[0x4d + i], scrambled_mod[i]);
        }

        // Step 2: XOR first 0x40 bytes with last 0x40 bytes
        for i in 0..0x40 {
            scrambled_mod[i] ^= scrambled_mod[0x40 + i];
        }

        // Step 3: XOR bytes 0x0d-0x10 with bytes 0x34-0x38
        for i in 0..4 {
            scrambled_mod[0x0d + i] ^= scrambled_mod[0x34 + i];
        }

        // Step 4: XOR last 0x40 bytes with first 0x40 bytes
        for i in 0..0x40 {
            scrambled_mod[0x40 + i] ^= scrambled_mod[i];
        }
        scrambled_mod
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn decrypt_data<T: AsRef<[u8]>>(&self, encrypted_data: T) -> anyhow::Result<BytesMut> {
        let data = encrypted_data.as_ref();
        let n = self.pair.0.n();
        let size = self.pair.0.size();
        let d = self.pair.0.d();

        // Convert ciphertext to BigUint
        let c = rsa::BigUint::from_bytes_be(data);

        // Perform modular exponentiation: c^d mod n
        let m = c.modpow(d, n);
        let mut decrypted_bytes = m.to_bytes_be();
        // padding
        if decrypted_bytes.len() < size {
            let pad_length = size - decrypted_bytes.len();
            decrypted_bytes.splice(0..0, vec![0; pad_length]); // Prepend padding
        }
        // Return the decrypted message as bytes
        Ok(BytesMut::from(Bytes::from(decrypted_bytes)))
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn from_pem_to_private(private_key_pem: &str) -> anyhow::Result<rsa::RsaPrivateKey> {
        // Parse the PEM string
        let pem = parse(private_key_pem).context("Failed to parse PEM string")?;

        // Ensure the PEM label is for an RSA public key
        // Ensure the PEM tag is for an RSA PRIVATE KEY
        if pem.tag() != "PRIVATE KEY" {
            bail!("Unexpected PEM tag: {}", pem.tag());
        }
        let private_key_der = rsa::RsaPrivateKey::from_pkcs8_der(pem.contents())
            .context("Failed to parse DER-encoded PKCS#8 private key")?;

        Ok(private_key_der)
    }
    #[allow(clippy::missing_errors_doc)]
    pub fn from_pem(private_key_pem: &str) -> anyhow::Result<Self> {
        let private_key_der = Self::from_pem_to_private(private_key_pem)?;
        let pub_key = private_key_der.to_public_key();
        Ok(Self::new((private_key_der, pub_key)))
    }
}

#[must_use]
#[allow(clippy::missing_panics_doc)]
pub fn generate_rsa_key_pair() -> (rsa::RsaPrivateKey, rsa::RsaPublicKey) {
    let bits: usize = 1024;
    let mut rng = rand::thread_rng();
    let private_key =
        rsa::RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate RSA private key");
    let public_key = rsa::RsaPublicKey::from(&private_key);
    (private_key, public_key)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::str::Trim;
    use rsa::pkcs1::{EncodeRsaPrivateKey, LineEnding};

    #[test]
    fn test_rsa_key_generated() {
        let keypair = generate_rsa_key_pair();
        let pem = keypair.0.to_pkcs1_pem(LineEnding::LF).unwrap();
        let formatted = String::from_utf8_lossy(pem.as_bytes());
        assert!(formatted.starts_with("-----BEGIN RSA PRIVATE KEY-----"));
    }

    #[test]
    fn test_encryption_works() {
        let modulus = [
            0, 223, 167, 200, 243, 159, 71, 142, 226, 187, 170, 69, 162, 8, 145, 92, 139, 207, 67,
            189, 1, 35, 109, 221, 188, 209, 20, 151, 56, 79, 70, 169, 46, 43, 166, 136, 99, 234, 1,
            212, 249, 191, 87, 41, 151, 102, 78, 192, 172, 57, 96, 199, 159, 204, 50, 5, 117, 148,
            85, 211, 203, 225, 211, 138, 173, 63, 12, 45, 94, 31, 14, 43, 248, 64, 85, 8, 55, 188,
            74, 101, 232, 218, 224, 185, 181, 248, 245, 201, 69, 133, 89, 95, 186, 28, 72, 54, 0,
            178, 194, 218, 96, 228, 6, 155, 52, 193, 24, 157, 192, 30, 84, 48, 0, 133, 76, 146, 83,
            185, 243, 100, 148, 180, 242, 5, 237, 62, 0, 159, 53,
        ];
        let unencrypted = [
            79, 112, 181, 90, 8, 119, 15, 29, 159, 106, 254, 130, 170, 198, 87, 88, 143, 86, 230,
            61, 98, 228, 151, 38, 253, 34, 225, 55, 105, 250, 215, 98, 30, 78, 104, 221, 149, 6,
            82, 128,
        ];
        let expected = [
            36, 35, 171, 97, 133, 151, 145, 133, 240, 167, 43, 52, 152, 194, 103, 216, 33, 116, 84,
            6, 190, 142, 175, 168, 229, 130, 203, 147, 67, 108, 231, 112, 238, 91, 171, 124, 254,
            82, 15, 244, 96, 58, 145, 240, 144, 91, 33, 247, 109, 186, 252, 4, 197, 62, 80, 172,
            130, 212, 35, 175, 34, 64, 209, 9, 182, 214, 116, 112, 254, 6, 175, 90, 180, 5, 215,
            32, 85, 132, 18, 230, 39, 165, 12, 149, 191, 100, 101, 100, 36, 107, 147, 37, 139, 193,
            145, 60, 93, 96, 156, 78, 103, 11, 3, 112, 243, 146, 241, 21, 6, 139, 5, 88, 144, 249,
            215, 81, 128, 96, 232, 226, 15, 60, 178, 15, 118, 38, 67, 109,
        ];
        let key = RSAPublicKey::from_modulus(&modulus).unwrap();
        let encrypted = key.encrypt(&BytesMut::from(&unencrypted[..]));
        assert_eq!(encrypted, expected, "Should encrypt properly");
    }

    #[test]
    fn test_test_request_auth_login() {
        let packet_bytes = &[
            0, 111, 125, 244, 145, 84, 105, 242, 208, 32, 190, 242, 250, 167, 184, 36, 251, 198,
            229, 162, 94, 164, 79, 87, 68, 170, 166, 176, 59, 40, 47, 27, 21, 25, 124, 150, 77, 89,
            181, 194, 116, 217, 110, 171, 209, 185, 77, 251, 96, 150, 93, 77, 252, 126, 12, 83,
            216, 199, 44, 212, 246, 101, 130, 122, 182, 243, 194, 146, 36, 40, 82, 243, 90, 25, 74,
            246, 47, 109, 37, 56, 212, 73, 43, 55, 160, 146, 76, 62, 32, 155, 81, 200, 83, 80, 74,
            192, 236, 142, 195, 1, 233, 42, 53, 176, 191, 251, 137, 116, 19, 216, 67, 43, 219, 71,
            199, 182, 215, 100, 56, 14, 72, 99, 39, 222, 240, 60, 93, 250, 227, 2, 137, 47, 122,
            247, 198, 200, 127, 195, 145, 4, 36, 217, 202, 40, 14, 60, 108, 223, 105, 93, 75, 251,
            208, 190, 162, 161, 229, 132, 42, 51, 87, 98, 80, 8, 186, 82, 88, 167, 103, 122, 13,
            195, 77, 123, 44, 220, 155, 160, 165, 190, 158, 33, 165, 66, 242, 21, 246, 171, 168,
            42, 84, 226, 106, 87, 18, 27, 148, 249, 170, 123, 122, 134, 21, 116, 104, 107, 61, 216,
            241, 249, 115, 160, 104, 100, 178, 171, 179, 221, 7, 232, 125, 192, 245, 13, 131, 39,
            207, 45, 123, 108, 196, 95, 55, 75, 104, 206, 89, 157, 39, 39, 156, 116, 100, 177, 248,
            92, 174, 21, 189, 35, 251, 208, 238, 82, 192, 125, 223, 53, 211, 170, 49, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 201, 60, 201, 172, 185, 36,
            197, 189, 152, 64, 89, 234, 166, 34, 61, 246, 0, 0, 0, 0, 97, 9, 131, 137, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let packet_body = &packet_bytes[1..];
        let (raw1, rest1) = packet_body.split_at(128);
        let (raw2, _) = rest1.split_at(128);
        let decr_expect = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 52, 0, 0, 97, 100, 109, 105, 110, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 97, 100, 109, 105, 110, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ];
        let scrambl =
            ScrambledRSAKeyPair::from_pem(include_str!("../../../test_data/test_private_key.pem"))
                .unwrap();
        let decr_raw1 = scrambl.decrypt_data(&BytesMut::from(raw1)).unwrap();
        let decr_raw2 = scrambl.decrypt_data(&BytesMut::from(raw2)).unwrap();
        let decrypted = [decr_raw1, decr_raw2].concat();
        assert_eq!(decrypted, decr_expect, "Should decrypted be ok");
        let part1 = String::from_utf8_lossy(&decrypted[0x4E..0x4E + 50]);
        let part2 = String::from_utf8_lossy(&decrypted[0xCE..0xCE + 14]);
        let result: String = format!("{}{}", part1.trim_all(), part2.trim_all());
        assert_eq!(result.to_string(), "admin");
    }
}
