
extern crate serde_json;

use core::traits::{ManagePasswords, StoreSessions, StoreUsers};
use core::errors::AppError;
use core::models::{Session, User};
use futures::future::{Future, err, ok};
use serde_json::from_slice;

#[derive(Serialize, Deserialize)]
pub struct BasicLoginRequest {
    email: String,
    password: String
}

#[derive(Serialize, Deserialize)]
pub struct BasicRegisterRequest {
    email: String,
    password: String
}

impl From<Vec<u8>> for BasicLoginRequest {
    fn from(body: Vec<u8>) -> BasicLoginRequest {
        from_slice(&body).unwrap()
    }
}

pub struct BasicAuthService {
    user_storage: Box<StoreUsers>,
    password_service: Box<ManagePasswords>,
    session_service: Box<StoreSessions>
}

impl BasicAuthService {
    fn new(user_storage: Box<StoreUsers>, password_service: Box<ManagePasswords>, session_service: Box<StoreSessions>) -> Self {
        Self {
            user_storage: user_storage,
            password_service: password_service,
            session_service: session_service
        }
    }
    
    pub fn authorize(&self, req: BasicLoginRequest) -> Box<Future<Item=Session, Error=AppError> + Send> {
        Box::new(self.user_storage.find_user_by_email(&req.email)
            .and_then(move | user| { self.validate_password(user, req.password) }))
    }  

    pub fn register(&self, req: BasicRegisterRequest) -> Box<Future<Item=User, Error=AppError> + Send> {
        let user = User::from(req);
        Box::new(self.user_storage.insert_user(user))
    }

    fn validate_password(&self, user: User, password: String) -> Box<Future<Item=Session, Error=AppError> + Send> {
        match self.password_service.check_password(&user.id, &password) {
            Ok(true) => self.session_service.create_session(user),
            Ok(false) => Box::new(err(AppError::Unauthorized)),
            Err(_) => Box::new(err(AppError::ApplicationError))
        }
    }
}