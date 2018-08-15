extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate uuid;
extern crate hyper_tls;
extern crate postgres;

extern crate crypto;
extern crate rustc_serialize as serialize;

use crypto::digest::Digest;
//use crypto::sha2::Sha256;
use serialize::base64::{STANDARD, ToBase64};
use uuid::Uuid;
use postgres::{Connection, TlsMode, params::ConnectParams};

use futures::future::*;
use futures::Stream;
use std::fmt::Debug;
use hyper::StatusCode;
use hyper::Uri;
use hyper::Error;
use hyper_tls::HttpsConnector;
#[macro_use]
extern crate serde_derive;

use hyper::{Body, Request, Response, Server, Method, service::Service};
use hyper::rt::Future;
use hyper::service::service_fn;

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();
    
    let server = Server::bind(&addr)
    .serve(|| { service_fn(handler) })
    .map_err(|e| eprintln!("server AppError: {}", e));

    hyper::rt::run(server);
}

fn handler(req: Request<Body>) -> impl Future<Item=Response<Body>, Error=hyper::Error> {
    parse(req)
    .and_then(handle)
}

fn parse(req: Request<Body>) -> impl Future<Item=RawRequest, Error=hyper::Error> {
    let method = req.method().clone();
    let target = req.uri().to_string();

    parse_body(req.into_body())
        .and_then(move |body| {
            ok(RawRequest { method, target, body })
        })
}

fn parse_body(body: Body) -> impl Future<Item=Vec<u8>, Error=hyper::Error> {
    
    body.fold(Vec::new(), |mut v, chunk| {
            v.extend(&chunk[..]);
            ok::<_, hyper::Error>(v)
        })
}

struct RawRequest {
    method: Method,
    target: String,
    body: Vec<u8>
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

impl std::convert::From<AppError> for hyper::Error {
    fn from(error: AppError) -> Self {
        panic!("I don't know what to do")
    }
}

fn error_to_response(error: AppError) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    Box::new(ok(Response::builder()
    .status(error.to_status())
    .body(Body::empty())
    .unwrap()
    ))
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

fn authorize_fb(req: FBLoginRequest) -> Box<Future<Item=Session, Error=AppError> + Send> {
    let url = format!("https://graph.facebook.com/me?access_token={}", req.token);
    
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
                .and_then(upsertUser)
                .and_then(create_session)
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

fn convert_and_parse(resp: Response<Body>) -> impl Future<Item=Vec<u8>, Error=hyper::Error> + Send {
    parse_body(resp.into_body())
}

fn upsertUser(user: User) -> impl Future<Item=User, Error=AppError> + Send {
    ok(user)
}

#[derive(Serialize, Deserialize)]
struct FBLoginRequest {
    token: String
}
impl FBLoginRequest {
    fn from(body: Vec<u8>) -> Result<FBLoginRequest, AppError> {
        Ok(serde_json::from_slice(&body).unwrap())
    }
}

fn successful_login(session: Session) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {

    Box::new(ok(Response::builder()
    .status(200)
    .body(Body::from(serde_json::to_string(&session).unwrap())).unwrap()))
}

#[derive(Serialize, Deserialize)]   
struct Session {
    id: String
}

fn authorize_basic(req: BasicLoginRequest) -> Box<Future<Item=Session, Error=AppError> + Send> {
    Box::new(find_user_by_email(&req.email)
    .and_then(move | user| { validate_password(user, req.password) }))
    
}  

fn validate_password(user: User, password: String) -> Box<Future<Item=Session, Error=AppError> + Send> {
    match password::check_password(&user.password, &password) {
        Ok(true) => create_session(user),
        Ok(false) => Box::new(err(AppError::Unauthorized)),
        Err(_) => Box::new(err(AppError::ApplicationError))
    }
}

pub mod password {
    
    use crypto::digest::Digest;
    use crypto::hmac::Hmac;
    use crypto::mac::Mac;
    use crypto::sha2::Sha256;
    use crypto::mac::MacResult;
    use serialize::base64::{STANDARD, ToBase64, Config, CharacterSet, Newline};
    pub fn check_password(hashed_password: &str, password: &str) -> Result<bool, super::AppError> {
        
        let mut sha = Sha256::new();
        let mut hmac = Hmac::new(sha, b"my key");

        hmac.input(password.as_bytes());

        let hash1 = hmac.result().code().to_base64(Config {char_set: CharacterSet::Standard, newline: Newline::CRLF, pad: true, line_length: Some(76)});
        println!("password supplied: {}, password in db: {}, hash: {}", password, hashed_password, hash1);
        
        match hash1 == hashed_password {
            true => { println!("User authorized"); Ok(true) },
            false => { println!("User NOT authorized"); Err(super::AppError::Unauthorized) }
        }
    }
}

fn create_session(user: User) -> Box<Future<Item=Session, Error=AppError> + Send> {
    let session = Session { id: uuid::Uuid::new_v4().to_string() };
    Box::new(match Connection::connect("postgres://postgres:1@localhost:5432/auth", TlsMode::None) {
        Ok(conn) => match &conn.execute("INSERT into public.session (session_id, user_id) values($1,$2)", &[ &session.id, &user.id ]) {
                Ok(_) => ok(session),
                Err(error) => { println!("Pos1{}", error); err(AppError::ApplicationError) }
            }
            Err(error) => { println!("Pos2{}", error); err(AppError::ApplicationError) }
    })
} 

fn find_user_by_email(email: &str) -> Box<Future<Item=User, Error=AppError> + Send> {
     
    
    Box::new(match Connection::connect("postgres://postgres:1@localhost:5432/auth", TlsMode::None) {
        
        Ok(conn) => match &conn.query("SELECT id, email, password FROM public.person where email = $1", &[ &email ]) {
            Ok(rows) => if !rows.is_empty() {
                    let row = rows.get(0);
                    println!("found user");
                    ok(User {
                        id: row.get("id"),
                        email: row.get("email"),
                        password: row.get("password")
                    })
                } else {
                    println!("no user found");
                    err(AppError::Unauthorized)
            },
            Err(error) => { println!("Pos1{}", error); err(AppError::ApplicationError) }
        }
        Err(error) => { println!("Pos2{}", error); err(AppError::ApplicationError) }
    })
}


#[derive(Serialize, Deserialize)]
pub enum AppError {
    ApplicationError,
    RoutingError,
    Unauthorized,
    BadRequest
}

impl AppError {
    fn from(statusCode: hyper::StatusCode) -> AppError {
        match statusCode {
            hyper::StatusCode::NOT_FOUND => AppError::RoutingError,
            hyper::StatusCode::FORBIDDEN => AppError::Unauthorized,
            _ => AppError::BadRequest
        }
    }

    fn to_status(&self) -> StatusCode {
    (match &self {
        AppError::ApplicationError => StatusCode::from_u16(500),
        AppError::RoutingError => StatusCode::from_u16(404),
        AppError::Unauthorized => StatusCode::from_u16(403),
        AppError::BadRequest => StatusCode::from_u16(400)
    }).unwrap()
}
}

impl Debug for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        Ok(())
    }
} 

 



struct User {
    pub id: String,
    pub email: String,
    pub password: String 
}

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
