extern crate redis;

use self::redis::Commands;
use core::traits::{MakeId, StoreSessions};
use core::models::{Session, User};
use core::errors::AppError;
use settings::RedisSettings;
use futures::future::{Future, ok, err};

pub struct RedisSessionService {
    uuid_maker: Box<MakeId>,
    settings: RedisSettings
}

impl RedisSessionService {
    pub fn new(settings: &RedisSettings, uuid_maker: Box<MakeId>) -> RedisSessionService {
        RedisSessionService {
            uuid_maker: uuid_maker,
            settings: settings.clone()
        }
    }
}

impl StoreSessions for RedisSessionService {
    fn create_session(&self, user: User) -> Box<Future<Item=Session, Error=AppError> + Send> {
        let session_id = self.uuid_maker.make_id();
        let connection_string = &self.settings.connection_string[..];
        let connection = redis::Client::open(connection_string)
        .and_then(get_connection)
        .and_then(|c| { c.set(user.id, session_id )}).unwrap();
        Box::new(ok(Session { id : session_id }))
    }
}

impl From<redis::RedisError> for AppError {
    fn from(error: redis::RedisError) -> AppError {
        AppError::ApplicationError
    }
}