pub  mod traits {
    
    use super::models::*;
    use super::errors::AppError;
    use futures::future::Future;

    pub trait Crypto {
        fn encrypt(password: &str) -> String;
    }

    pub trait ManagePasswords {
        fn set_password(userId: &str, password: &str) -> Future<Item=(), Error=AppError> + Send;
        fn check_password(userId: &str, password: &str) -> Future<Item=bool, Error=AppError> + Send;
    }

    pub trait StoreUsers {
        fn find_user_by_id(id: &str) -> Future<Item=Option<User>, Error=AppError> + Send;
        fn insert_user(user: User) -> Future<Item=User, Error=AppError> + Send;
    }

    pub trait StoreSessions {
        fn create_session(user: User) -> Future<Item=Session, Error=AppError> + Send;
    }

    pub trait MakeId {
        fn make_id() -> String;

    }
}

pub mod models {
    use hyper::Method;

    #[derive(Serialize, Deserialize)]   
    pub struct User {
        pub id: String,
        pub name: String
    }

    #[derive(Serialize, Deserialize)]   
    pub struct Session {
        id: String
    }

    pub struct RawRequest {
        method: Method,
        target: String,
        body: Vec<u8>
    }   
}

pub mod errors {
    extern crate hyper;
    extern crate std;
    use hyper::StatusCode;

    #[derive(Serialize, Deserialize)]
    pub enum AppError {
        ApplicationError,
        RoutingError,
        Unauthorized,
        BadRequest
    }

    impl From<hyper::StatusCode> for AppError {
        fn from(statusCode: hyper::StatusCode) -> AppError {
            match statusCode {
                hyper::StatusCode::NOT_FOUND => AppError::RoutingError,
                hyper::StatusCode::FORBIDDEN => AppError::Unauthorized,
                _ => AppError::BadRequest
            }
        }
    }

    impl From<AppError> for hyper::Error {
        fn from(error: AppError) -> Self {
            panic!("I don't know what to do")
        }
    }

    impl Debug for AppError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            Ok(())
        }
    }

    impl AppError {
        pub fn to_status(&self) -> StatusCode {
            (match &self {
                    AppError::ApplicationError => StatusCode::from_u16(500),
                    AppError::RoutingError => StatusCode::from_u16(404),
                    AppError::Unauthorized => StatusCode::from_u16(403),
                    AppError::BadRequest => StatusCode::from_u16(400)
                }).unwrap()
        }
    }
}