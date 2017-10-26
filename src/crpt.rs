use errors::{Error};
use utils::*;
use rand::{OsRng, Rng};
use crypto::chacha20::ChaCha20;
use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::poly1305::Poly1305;
use crypto::mac::{Mac, MacResult};
use std::ops::Range;
use byteorder::{ByteOrder, BigEndian};

struct RegistryFile {
    pub decrypted_len: usize,
    pub key_salt_len: usize,
    pub chacha_nonce_len: usize,
    pub tag_len: usize,
    pub bcrypt_pbkdf_cost: u32
}

impl RegistryFile {
    pub fn new_with_decrypted_length(len: usize) -> Self {
        RegistryFile {
            decrypted_len: len,
            key_salt_len: 16,
            chacha_nonce_len: 8,
            tag_len: 16,
            bcrypt_pbkdf_cost: 16
        }
    }

    pub fn new_with_encrypted_length(len: usize) -> Result<Self, Error> {
        if len < 44 {
            return Err(Error::CorruptedFileContent)
        }
        Ok(RegistryFile {
            decrypted_len: len - 44,
            key_salt_len: 16,
            chacha_nonce_len: 8,
            tag_len: 16,
            bcrypt_pbkdf_cost: 16
        })
    }

    pub fn encrypt(&self, body: &[u8], password: &str) -> Vec<u8> {
        let mut content: Vec<u8> = vec![0u8; self.encrypted_len()];
        self.fill_header(&mut content[..]);
        let mut key: Vec<u8> = build_key(password, &content[self.key_salt()], self.bcrypt_pbkdf_cost);
        self.encrypt_body(&key[..], &mut content[..], body);
        self.fill_mac(&key[..], &mut content[..]);
        content
    }

    pub fn decrypt(&self, encrypted: &Vec<u8>, password: &str) -> Result<Vec<u8>, Error> {
        let mut key = self.extract_key(&encrypted[..], password);
        self.validate_mac(&key[..], &encrypted[..])?;
        Ok(self.decrypt_body(&key[..], &encrypted[..]))
    }

    fn extract_key(&self, encrypted: &[u8], password: &str) -> Vec<u8> {
        let key_salt = &encrypted[self.key_salt()] as &[u8];
        let bcrypt_pbkdf_cost = BigEndian::read_u32(&encrypted[self.bcrypt_pbkdf()]);
        build_key(password, key_salt, bcrypt_pbkdf_cost)
    }

    fn validate_mac(&self, key: &[u8], encrypted: &[u8]) -> Result<(), Error> {
        let body = &encrypted[self.body()] as &[u8];
        let tag = &encrypted[self.tag()] as &[u8];
        let mac = MacResult::new(&tag);
        let mut poly1305 = Poly1305::new(&key[32..64]);
        poly1305.input(&body);
        if !poly1305.result().eq(&mac) {
            return Err(Error::WrongPassword);
        }
        Ok(())
    }

    fn decrypt_body(&self, key: &[u8], encrypted: &[u8]) -> Vec<u8> {
        let mut decrypted_body: Vec<u8> = vec![0u8; self.decrypted_len];
        if self.decrypted_len > 0 {
            let body = &encrypted[self.body()] as &[u8];
            let mut chacha = ChaCha20::new(&key[..32], &encrypted[self.chacha_nonce()]);
            chacha.process(&body, &mut decrypted_body[..]);
        }
        decrypted_body
    }

    fn fill_header(&self, content: &mut [u8]) {
        let mut gen = OsRng::new().expect("Failed to get OS random generator");
        gen.fill_bytes(&mut content[self.key_salt()]);
        BigEndian::write_u32(&mut content[self.bcrypt_pbkdf()], self.bcrypt_pbkdf_cost);
        gen.fill_bytes(&mut content[self.chacha_nonce()]);
    }

    fn encrypt_body(&self, key: &[u8], content: &mut [u8], input: &[u8]) {
        if self.decrypted_len > 0 {
            let mut chacha = ChaCha20::new(&key[..32], &content[self.chacha_nonce()]);
            chacha.process(input, &mut content[self.body()]);
        }
    }

    fn fill_mac(&self, key: &[u8], content: &mut [u8]) {
        let mut poly1305 = Poly1305::new(&key[32..64]);
        poly1305.input(&content[self.body()]);
        let mac = poly1305.result();
        copy_memory(mac.code(), &mut content[self.tag()]);
    }

    fn key_salt(&self) -> Range<usize> {
        (0..self.key_salt_len)
    }

    fn bcrypt_pbkdf(&self) ->  Range<usize> {
        let start = self.key_salt().end;
        (start..(start + 4))
    }

    fn chacha_nonce(&self) -> Range<usize> {
        let start = self.bcrypt_pbkdf().end;
        (start..(start + self.chacha_nonce_len))
    }

    fn body(&self) ->  Range<usize> {
        let start = self.chacha_nonce().end;
        (start..(start + self.decrypted_len))
    }

    fn tag(&self) ->  Range<usize> {
        let start = self.body().end;
        (start..(start + self.tag_len))
    }

    fn encrypted_len(&self) -> usize {
        self.tag().end
    }
}


fn build_key(password: &str, salt: &[u8], rounds: u32) -> Vec<u8> {
    use crypto::bcrypt_pbkdf::bcrypt_pbkdf;
    let mut output: Vec<u8> = vec![0u8; 64];
    bcrypt_pbkdf(password.as_bytes(), salt, rounds, &mut output);
    output
}

pub fn encrypt(body: &[u8], password: &str) -> Vec<u8> {
    let file = RegistryFile::new_with_decrypted_length(body.len());
    file.encrypt(body, password)
}

pub fn decrypt(encrypted: &Vec<u8>, password: &str) -> Result<Vec<u8>, Error> {
    let file = RegistryFile::new_with_encrypted_length(encrypted.len())?;
    file.decrypt(encrypted, password)
}