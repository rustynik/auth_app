#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct BasicLoginRequest {
    email: String,
    password: String
}

impl BasicLoginRequest {
    fn from(body: Vec<u8>) -> Result<BasicLoginRequest, AppError> {
        Ok(serde_json::from_slice(&body).unwrap())
    }
}

pub struct BasicAuthHandler {
    db: PostgresDb,
    password_service: PasswordService,
    session_service: SessionService
}

impl BasicAuthHandler {
    pub fn new(db: PostgresDb, password_service: PasswordService) -> Self {
        Self {
            db: PostgresDb,
            password_service: PasswordService,
            session_service: SessionService
        }
    }
    
    pub fn authorize_basic(&self.req: BasicLoginRequest) -> Box<Future<Item=Session, Error=AppError> + Send> {
        Box::new(self.db.find_user_by_email(&req.email)
            .and_then(move | user| { validate_password(user, req.password) }))
    }  

    fn validate_password(user: User, password: String) -> Box<Future<Item=Session, Error=AppError> + Send> {
        match password_service.check_password(&user.password, &password) {
            Ok(true) => session_service.create_session(user),
            Ok(false) => Box::new(err(AppError::Unauthorized)),
            Err(_) => Box::new(err(AppError::ApplicationError))
        }
    }