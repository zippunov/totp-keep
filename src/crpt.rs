use errors::{Error};
use utils::*;
use rand::{OsRng, Rng};
use crypto::bcrypt_pbkdf::bcrypt_pbkdf;
use crypto::chacha20::ChaCha20;
use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::poly1305::Poly1305;
use crypto::mac::{Mac, MacResult};
use std::ops::Range;
use byteorder::{ByteOrder, BigEndian};

pub struct RegistryFile {
    pub key_salt_len: usize,
    pub chacha_nonce_len: usize,
    pub tag_len: usize,
    pub bcrypt_pbkdf_cost: u32
}

impl RegistryFile {
    pub fn new() -> Self {
        RegistryFile{
            key_salt_len: 16,
            chacha_nonce_len: 8,
            tag_len: 16,
            bcrypt_pbkdf_cost: 16
        }
    }

    pub fn key_salt(&self) -> Range<usize> {
        (0..self.key_salt_len)
    }

    pub fn bcrypt_pbkdf(&self) ->  Range<usize> {
        let start = self.key_salt().end;
        (start..(start + 4))
    }

    pub fn chacha_nonce(&self) -> Range<usize> {
        let start = self.bcrypt_pbkdf().end;
        (start..(start + self.chacha_nonce_len))
    }

    pub fn body(&self, decrypted_len: usize) ->  Range<usize> {
        let start = self.chacha_nonce().end;
        (start..(start + decrypted_len))
    }

    pub fn tag(&self, decrypted_len: usize) ->  Range<usize> {
        let start = self.body(decrypted_len).end;
        (start..(start + self.tag_len))
    }

    pub fn encrypted_len(&self, decrypted_len: usize) -> usize {
        self.tag(decrypted_len).end
    }

    pub fn decrypted_len(&self, encrypted_len: usize) -> Result<usize, Error> {
        if encrypted_len < (self.chacha_nonce().end + self.tag_len) {
            return Err(Error::CorruptedFileContent);
        }
        Ok(encrypted_len - self.chacha_nonce().end - self.tag_len)
    }
}


fn build_key(password: &str, salt: &[u8], rounds: u32) -> Vec<u8> {
    use crypto::bcrypt_pbkdf::bcrypt_pbkdf;
    let mut output: Vec<u8> = vec![0u8; 64];
    bcrypt_pbkdf(password.as_bytes(), salt, rounds, &mut output);
    output
}

pub fn encrypt(body: &[u8], password: &str) -> Vec<u8> {
    let decrypted_len = body.len();
    let file = RegistryFile::new();
    let mut result: Vec<u8> = vec![0u8; file.encrypted_len(decrypted_len)];

    let mut gen = OsRng::new().expect("Failed to get OS random generator");
    gen.fill_bytes(&mut result[file.key_salt()]);
    BigEndian::write_u32(&mut result[file.bcrypt_pbkdf()], file.bcrypt_pbkdf_cost);
    gen.fill_bytes(&mut result[file.chacha_nonce()]);

    // key 64 bytes.
    // - First 32 bytes makes key for the Chahca20,
    // - Second 32 bytes makes key for poly1305
    let mut key: Vec<u8> = build_key(password, &result[file.key_salt()], file.bcrypt_pbkdf_cost);
    let chacha20_key_range = (..32);
    let poly1305_key_range = (32..64);

    if decrypted_len > 0 {
        let mut chacha = ChaCha20::new(&key[chacha20_key_range], &result[file.chacha_nonce()]);
        chacha.process(body, &mut result[file.body(decrypted_len)]);
    }

    let mut poly1305 = Poly1305::new(&key[poly1305_key_range]);
    poly1305.input(&result[file.body(decrypted_len)]);
    let mac = poly1305.result();
    copy_memory(mac.code(), &mut result[file.tag(decrypted_len)]);
    result
}

pub fn decrypt(encrypted: &Vec<u8>, password: &str) -> Result<Vec<u8>, Error> {
    let file = RegistryFile::new();
    let decrypted_len = file.decrypted_len(encrypted.len())?;
    let key_salt = &encrypted[file.key_salt()];
    let bcrypt_pbkdf_cost = BigEndian::read_u32(&encrypted[file.bcrypt_pbkdf()]);
    let mut key: Vec<u8> = build_key(password, key_salt, bcrypt_pbkdf_cost);
    let body = &encrypted[file.body(decrypted_len)];
    let tag = &encrypted[file.tag(decrypted_len)];
    let chacha20_key_range = (..32);
    let poly1305_key_range = (32..64);

    let mac = MacResult::new(&tag);
    let mut poly1305 = Poly1305::new(&key[poly1305_key_range]);
    poly1305.input(&body);
    if !poly1305.result().eq(&mac) {
        return Err(Error::WrongPassword);
    }
    let mut decrypted_body: Vec<u8> = vec![0u8; decrypted_len];
    if decrypted_len > 0 {
        let mut chacha = ChaCha20::new(&key[chacha20_key_range], &encrypted[file.chacha_nonce()]);
        chacha.process(&body, &mut decrypted_body[..]);
    }
    Ok(decrypted_body)
}