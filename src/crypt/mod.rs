pub mod constants;
pub mod login;
pub mod new;
pub mod rsa;
pub static STATIC_BLOWFISH_KEY: [u8; 16] = [154, 125, 7, 25, 132, 212, 137, 240, 220, 37, 6, 180, 21, 131, 47, 197];

use rand::{thread_rng, Rng};
pub const BLOWFISH_KEY_SIZE: usize = 16;

pub fn generate_blowfish_key() -> [u8; BLOWFISH_KEY_SIZE] {
    let mut key = [0u8; BLOWFISH_KEY_SIZE];
    let mut rng = thread_rng();
    for item in key.iter_mut().take(BLOWFISH_KEY_SIZE) {
        *item = rng.gen();
    }
    key
}
