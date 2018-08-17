

// TODO: Use TOML?

extern crate serde_json;

use std::fs::File;


/// application settings 
/// TODO: this implementation is fixed and does not really provide for varying underlying services 

#[derive(Deserialize, Debug, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub password: CryptoSettings,
    pub postgres: PostgresSettings 
}

#[derive(Deserialize, Debug, Clone)]
pub struct CryptoSettings { 
    crypto_key: String,
    salt: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct PostgresSettings {
    pub host : String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub db_name: String,
    pub schema_name: String
}

pub fn read(path : &str) -> ApplicationSettings {

    let rdr = File::open(path)
        .expect(&format!("Application settings file should be available at path {}", path));

    let settings : ApplicationSettings = serde_json::from_reader(rdr)
        .expect(&format!("Non-JSON file or incorrect application settings in file {}", path));

    settings
}   