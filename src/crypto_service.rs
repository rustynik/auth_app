extern crate crypto;
extern crate rustc_serialize as serialize;

use self::crypto::digest::Digest;
use self::crypto::hmac::Hmac;
use self::crypto::mac::Mac;
use self::crypto::sha2::Sha256;
use self::crypto::mac::MacResult;
use self::serialize::base64::{STANDARD, ToBase64, Config, CharacterSet, Newline};
use settings::CryptoSettings;

use core::traits::Crypto;

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
    fn encrypt_password(&self, password: &str) -> String {
        let mut sha = Sha256::new();
        let mut hmac = Hmac::new(sha, self.key);

        hmac.input(password.as_bytes().push(self.salt));

        hmac.result().code().to_base64(Config {char_set: CharacterSet::Standard, newline: Newline::CRLF, pad: true, line_length: Some(76)})
    }
}