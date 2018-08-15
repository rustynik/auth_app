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
mod conversions;
mod errors;
mod fb_auth_handler;

use requests::*;
use conversions::*;


fn main() {
    let app_settings = settings::read(&resolve_settings::resolve_settings_path());
    let addr = ([127, 0, 0, 1], app_settings.port).into();
    
    let server = Server::bind(&addr)
    .serve(|| { service_fn(handler) })
    .map_err(|e| eprintln!("server AppError: {}", e));

    hyper::rt::run(server);
}

fn handler(req: Request<Body>) -> impl Future<Item=Response<Body>, Error=hyper::Error> {
    parse(req)
    .and_then(handle)
}

 

 

fn handle(req: RawRequest) -> impl Future<Item=Response<Body>, Error=hyper::Error> {
    (match (req.method, req.target.as_str(), req.body) {
        (Method::POST, "/login", body) => handle_basic_login(body),
        (Method::POST, "/login/fb", body) => handle_fb_login(body),
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



 



fn handle_basic_login(body: Vec<u8>) -> Box<Future<Item=Session, Error=AppError> + Send> {
    let basic_login_request = BasicLoginRequest::from(body).unwrap();
    println!("login {} {}", basic_login_request.email, basic_login_request.password);
    authorize_basic(basic_login_request)
}

fn handle_fb_login(body: Vec<u8>) -> Box<Future<Item=Session, Error=AppError> + Send> {
    let req = FBLoginRequest::from(body).unwrap();
    println!("login {}", req.token);
    authorize_fb(req)
}
