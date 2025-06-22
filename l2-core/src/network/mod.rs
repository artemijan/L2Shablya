use crate::crypt::login::Encryption;
use bytes::BytesMut;

pub mod connection;
pub mod connector;
pub mod listener;

pub fn encrypt_internal_packet(bytes: &mut BytesMut, blowfish: &Encryption) {
    let size = bytes.len();
    Encryption::append_checksum(&mut bytes[2..size]);
    blowfish.encrypt(&mut bytes[2..size]);
}
