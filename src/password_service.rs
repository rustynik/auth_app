extern crate crypto;
extern crate rustc_serialize as serialize;

use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use crypto::mac::MacResult;
use serialize::base64::{STANDARD, ToBase64, Config, CharacterSet, Newline};

pub struct PasswordService {
    key: Vec<u8>,
    salt: Vec<u8>
}

impl  PasswordService {
    pub fn new(key: Vec<u8>, salt: Vec<u8>) -> Self {
        Self {
            key: key,
            salt: salt
        }
    }

    pub fn check_password(hashed_password: &str, password: &str) -> Result<bool, super::AppError> {
        
        let hash1 = encrypt_password(password);

        println!("password supplied: {}, password in db: {}, hash: {}", password, hashed_password, hash1);
        
        match hash1 == hashed_password {
            true => { println!("User authorized"); Ok(true) },
            false => { println!("User NOT authorized"); Err(super::AppError::Unauthorized) }
        }
    }

    pub fn encrypt_password(password: &str) -> Vec<u8> {
        let mut sha = Sha256::new();
        let mut hmac = Hmac::new(sha, key);

        hmac.input(password.as_bytes().push(salt));

        hmac.result().code().to_base64(Config {char_set: CharacterSet::Standard, newline: Newline::CRLF, pad: true, line_length: Some(76)})
    }
}