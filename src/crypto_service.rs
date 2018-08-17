extern crate crypto;
extern crate rustc_serialize as serialize;

use crypto::digest::Digest;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use crypto::mac::MacResult;
use serialize::base64::{STANDARD, ToBase64, Config, CharacterSet, Newline};

use core::Crypto;

pub struct CryptoService {
    key: Vec<u8>,
    salt: Vec<u8>
}

impl CryptoService {
    pub fn new(settings: &CryptoSettings) -> Self {
        Self {
            key: settings.key.clone().as_bytes(),
            key: settings.salt.clone().as_bytes()
        }
    }
}

impl Crypto for CryptoService {
    pub fn encrypt_password(password: &str) -> String {
        let mut sha = Sha256::new();
        let mut hmac = Hmac::new(sha, key);

        hmac.input(password.as_bytes().push(salt));

        hmac.result().code().to_base64(Config {char_set: CharacterSet::Standard, newline: Newline::CRLF, pad: true, line_length: Some(76)})
    }
}