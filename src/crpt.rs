use errors::{Error};
use utils::*;
use rand::{OsRng, Rng};
use crypto::bcrypt_pbkdf::bcrypt_pbkdf;
use crypto::chacha20::ChaCha20;
use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::poly1305::Poly1305;
use crypto::mac::{Mac, MacResult};


fn build_key(password: &str, salt: &[u8]) -> Vec<u8> {
    use crypto::bcrypt_pbkdf::bcrypt_pbkdf;
    let mut key: Vec<u8> = vec![0u8; 32];
    bcrypt_pbkdf(password.as_bytes(), salt, 16, &mut key);
    key
}

pub fn encrypt(body: &[u8], password: &str) -> Vec<u8> {
    let len = body.len();
    let mut result: Vec<u8> = vec![0u8; len + 32];
    let mut gen = OsRng::new().expect("Failed to get OS random generator");
    gen.fill_bytes(&mut result[..16]);
    let mut key: Vec<u8> = build_key(password, &result[..16]);
    if len > 0 {
        let mut chacha = ChaCha20::new(&key, &result[..8]);
        chacha.process(body, &mut result[16..(len + 16)]);
    }
    let mut poly1305 = Poly1305::new(&key);
    poly1305.input(&result[16..(len + 16)]);
    let mac = poly1305.result();
    copy_memory(mac.code(), &mut result[(len + 16)..]);
    result
}

pub fn decrypt(encrypted: &Vec<u8>, password: &str) -> Result<Vec<u8>, Error> {
    let len = encrypted.len() - 32;
    if len < 0 {
        return return Err(Error::CorruptedFileContent);
    }
    let salt = &encrypted[..16];
    let body = &encrypted[16..(len + 16)];
    let tag = &encrypted[(len + 16)..];
    let mut key: Vec<u8> = build_key(password, salt);
    let mac = MacResult::new(&tag);
    let mut poly1305 = Poly1305::new(&key[..]);
    poly1305.input(&body);
    if !poly1305.result().eq(&mac) {
        return Err(Error::WrongPassword);
    }
    let mut decrypted_body: Vec<u8> = vec![0u8; len];
    if len > 0 {
        let mut chacha = ChaCha20::new(&key, &encrypted[..8]);
        chacha.process(&body, &mut decrypted_body[..]);
    }
    Ok(decrypted_body)
}