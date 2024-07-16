use crate::crypt::blowfish_engine::STATIC_BLOWFISH_KEY;
use crate::crypt::new_crypt::NewCrypt;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct LoginEncryption {
    pub crypt: NewCrypt,
    pub static_crypt: NewCrypt,
}

impl LoginEncryption {
    pub fn new(key: &[u8]) -> Self {
        LoginEncryption {
            crypt: NewCrypt::from_u8_key(key),
            static_crypt: NewCrypt::from_u8_key(&STATIC_BLOWFISH_KEY.clone()),
        }
    }
}
