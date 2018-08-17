extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate hyper_tls;
extern crate postgres;

mod password_service;
mod uuid_service;


use postgres::{Connection, TlsMode, params::ConnectParams};

use futures::future::*;
use futures::Stream;
use std::fmt::Debug;
use hyper::StatusCode;
use hyper::Uri;
use hyper::Error;
use hyper_tls::HttpsConnector;


use hyper::{Body, Request, Response, Server, Method, service::Service};
use hyper::rt::Future;
use hyper::service::service_fn;

mod settings;
mod resolve_settings;
mod requests;
mod errors;
mod fb_auth_service;
mod basic_auth_service;

use settings::ApplicationSettings;
use requests::{RawRequest, parse};
use conversions;
use fb_auth_service::{FBLoginRequest};
use basic_auth_service::{BasicLoginRequest};
use std::sync::{Arc, Mutex};
use errors::AppError;

type SharedSettings = Arc<Mutex<ApplicationSettings>>;

/// application entry point
fn main() {
    let app_settings = settings::read(&helpers::resolve_settings::resolve_settings_path());
    
    init_db(&app_settings.postgres);

    let app_settings = Arc::new(Mutex::new(app_settings));

    let addr = ([127, 0, 0, 1], app_settings.port).into();

    let server = Server::bind(&addr)
        .serve(|| { service_fn(move |req| {
                parse(req).and_then(|req| { route(req, app_settings.clone()) })
            }) 
        })
        .map_err(|e| eprintln!("server AppError: {}", e));

    hyper::rt::run(server);
}

/// route request to a handler 
/// and respond by returning either a session or a http error status code
fn route(req: RawRequest, settings: SharedSettings) -> impl Future<Item=Response<Body>, Error=hyper::Error> {
    
    let settings = &*(settings.lock().unwrap());
    
    // no router is required for this prototype server, but a more sophisticated implementation 
    // would probably need a real router and an abstraction over the auth services
    // to plugin with different request urls
    (match (req.method, req.target.as_str(), req.body) {
        
        (Method::POST, "/login", body) => {
            let req = BasicLoginRequest::from(body).unwrap();  
            helpers::create_basic_auth_service(settings).authorize(req) 
        },

        (Method::POST, "/login/fb", body) => {
            let req = FBLoginRequest::from(body).unwrap();
            println!("login {}", req.token);
            helpers::create_fb_auth_service(settings).authorize(req)
        },

        _ => Box::new(err(AppError::RoutingError))
    })
    .then(|res| {
        match res {
            Ok(session) => successful_login(session),
            Err(error) => error_to_response(error)
        }
    })
    .from_err()
}

mod helpers {
    
    use std::env;

    pub fn resolve_settings_path() -> String {
        let args: Vec<String> = env::args().collect();

        match args.len() {
            2 => args[1].to_owned(),
            _ => {
                let mut dir = env::current_exe().expect("Cannot get current directory");
                dir.set_file_name("config.json");
                dir.to_str().expect("Invalid path").to_owned()
            }
        }
    }
    
    pub fn create_basic_auth_service(settings: &ApplicationSettings) => BasicAuthService {
        let user_storage = postgres_db::create_user_storage(&settings.postgres);
        let password_storage = postgres_db::create_password_storage(&settings.password);
        let session_service = redis_db::create_session_service(&settings.session);

        BasicAuthService::new(user_storage, password_storage, session_service)
    }

    pub fn create_fb_auth_service(settings: &ApplicationSettings) -> FbAuthService {    
        let user_storage = postgres_db::create_user_storage(&settings.postgres);
        let session_service = redis_db::create_session_service(&settings.session);

        FbAuthService::new(user_storage, session_service)
    }
}