pub mod login;
pub mod rsa;
pub static STATIC_BLOWFISH_KEY: [u8; 16] = [
    154, 125, 7, 25, 132, 212, 137, 240, 220, 37, 6, 180, 21, 131, 47, 197,
];

use rand::{thread_rng, Rng};
pub const BLOWFISH_KEY_SIZE: usize = 16;

#[must_use]
pub fn generate_blowfish_key(size: Option<usize>) -> Vec<u8> {
    let the_size = size.unwrap_or(BLOWFISH_KEY_SIZE);
    let mut key = vec![0u8; the_size];
    let mut rng = thread_rng();
    for item in key.iter_mut().take(the_size) {
        *item = rng.gen();
    }
    key
}
