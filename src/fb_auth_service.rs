extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use postgres_db;
use password_service;
use session_service;
use serde_json::from_slice;

const fb_url : &str = "https://graph.facebook.com/me?access_token={}";

#[derive(Serialize, Deserialize)]
pub struct FBLoginRequest {
    token: String
}

impl FBLoginRequest {
    pub fn from(body: Vec<u8>) -> Result<FBLoginRequest, AppError> {
        Ok(serde_json::from_slice(&body).unwrap())
    }
}

pub fn create_service(settings: &ApplicationSettings) -> FbAuthService {
    
    let db = postgres_db::create_service(settings);
    let session_service = session_service::create_service(settings);

    FbAuthService::new(db, session_service)
}


pub struct FbAuthService {
    db: PostgresDb, 
    session_service: SessionService
}

impl FbAuthService {
    fn new(db: PostgresDb, session_service: SessionService) -> Self {
        Self {
            db: db,
            session_service: session_service
        }
    }

    pub fn authorize(&self, req: FBLoginRequest) -> Box<Future<Item=Session, Error=AppError> + Send> {
    
        let url = format!(fb_url, req.token);
    
        match url.as_str().parse::<Uri>() {
            Ok(uri) => {
                let https = HttpsConnector::new(4).unwrap();
                let client = hyper::Client::builder().build::<_, hyper::Body>(https);
                
                println!("checking fb auth token, url: {}", uri);

                Box::new(
                client
                    .get(uri)
                    .map_err(|error| { 
                        // TOOD: to AppError
                        match error.into_cause() {
                            Some(err) => println!("{}", err.description()),
                            None => println!("хрен разберет")
                        } 
                        AppError::ApplicationError
                    })
                    .and_then(check_status)
                    .and_then(get_fb_user_data)
                    .and_then(self.db.upsertUser)
                    .and_then(self.session_service.create_session) // this may probably be extracted to an upper level? 
                )
            },
            Err(err) => { 
                println!("fb response failed, {}", err);
                Box::new(futures::future::err(AppError::BadRequest))
            }
        }
    }

    fn check_status(resp: Response<Body>) -> impl Future<Item=Response<Body>, Error=AppError> + Send {
        println!("{}", resp.status());
        match resp.status() {
            hyper::StatusCode::OK => ok(resp),
            error => err(AppError::from(error))
        }
    }

    fn get_fb_user_data(resp: Response<Body>) -> impl Future<Item=User, Error=AppError> + Send {

        convert_and_parse(resp)
            .map_err(|err| AppError::ApplicationError)
            .and_then(|parsed| {
                let fb_data : serde_json::Value = serde_json::from_slice(&parsed).unwrap();
                ok(User { id: fb_data["id"].to_string(), email: fb_data["email"].to_string(), password: "s".to_string() })
            })
    }

    
}

