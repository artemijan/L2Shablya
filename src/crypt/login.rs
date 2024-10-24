use crate::crypt::STATIC_BLOWFISH_KEY;
use crate::crypt::new::Crypt;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Encryption {
    pub crypt: Crypt,
    pub static_crypt: Crypt,
}

impl Encryption {
    pub fn new(key: &[u8]) -> Self {
        Encryption {
            crypt: Crypt::from_u8_key(key),
            static_crypt: Crypt::from_u8_key(&STATIC_BLOWFISH_KEY.clone()),
        }
    }
}
