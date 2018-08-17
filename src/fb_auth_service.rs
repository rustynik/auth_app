extern crate serde_json;
extern crate hyper;
extern crate hyper_tls;
extern crate futures;

use futures::future::{Future, err, ok};
use core::errors::AppError;
use core::traits::{StoreSessions, StoreUsers};
use core::models::{Session, User};
use hyper::{Client, Uri, Body, Response};
use serde_json::from_slice;
use hyper_tls::HttpsConnector;
use super::requests::*;

#[derive(Serialize, Deserialize)]
pub struct FBLoginRequest {
    pub token: String
}

impl From<Vec<u8>> for FBLoginRequest {
    fn from(body: Vec<u8>) -> FBLoginRequest {
        from_slice(&body).unwrap()
    }
}

pub struct FbAuthService {
    user_storage: Box<StoreUsers>, 
    session_service: Box<StoreSessions>
}

impl FbAuthService {
    fn new(user_storage: Box<StoreUsers>, session_service: Box<StoreSessions>) -> Self {
        Self {
            user_storage: user_storage,
            session_service: session_service
        }
    }

    pub fn authorize(&self, req: FBLoginRequest) -> Box<Future<Item=Session, Error=AppError> + Send> {
    
        let url = format!("https://graph.facebook.com/me?access_token={}", req.token);
    
        match url.as_str().parse::<Uri>() {
            Ok(uri) => {
                let https = HttpsConnector::new(4).unwrap();
                let client = Client::builder().build::<_, Body>(https);
                
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
                    .and_then(&self.check_status)
                    .and_then(&self.get_fb_user_data)
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

    fn check_status(&self, resp: Response<Body>) -> impl Future<Item=Response<Body>, Error=AppError> + Send {
        println!("{}", resp.status());
        match resp.status() {
            hyper::StatusCode::OK => ok(resp),
            error => err(AppError::from(error))
        }
    }

    fn get_fb_user_data(&self, resp: Response<Body>) -> impl Future<Item=User, Error=AppError> + Send {

        convert_and_parse(resp)
            .map_err(|err| AppError::ApplicationError)
            .and_then(|parsed| {
                let fb_data : serde_json::Value = serde_json::from_slice(&parsed).unwrap();
                ok(User { id: fb_data["id"].to_string(), name: fb_data["name"].to_string() })
            })
    }

    
}

