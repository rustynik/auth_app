fn error_to_response(error: AppError) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {
    Box::new(ok(Response::builder()
    .status(error.to_status())
    .body(Body::empty())
    .unwrap()
    ))
}

fn successful_login(session: Session) -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send> {

    Box::new(ok(Response::builder()
    .status(200)
    .body(Body::from(serde_json::to_string(&session).unwrap())).unwrap()))
}