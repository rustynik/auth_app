pub struct RawRequest {
    method: Method,
    target: String,
    body: Vec<u8>
}

pub fn parse(req: Request<Body>) -> impl Future<Item=RawRequest, Error=hyper::Error> {
    let method = req.method().clone();
    let target = req.uri().to_string();

    parse_body(req.into_body())
        .and_then(move |body| {
            ok(RawRequest { method, target, body })
        })
}

pub fn parse_body(body: Body) -> impl Future<Item=Vec<u8>, Error=hyper::Error> {
    
    body.fold(Vec::new(), |mut v, chunk| {
            v.extend(&chunk[..]);
            ok::<_, hyper::Error>(v)
        })
}

pub fn convert_and_parse(resp: Response<Body>) -> impl Future<Item=Vec<u8>, Error=hyper::Error> + Send {
    parse_body(resp.into_body())
}