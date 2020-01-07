use hyper::{Request, Body, Response, Uri};
use hyper::client::{HttpConnector, Client};
use std::error::Error;
use http::uri::PathAndQuery;
use hyper::header::HOST;
use lazy_static::lazy_static;
use hyper_tls::HttpsConnector;

use log::info;

const TARGET_SCHEME: &str = "https";
const TARGET_HOST: &str = "ya.ru";

lazy_static! {
 static ref CLIENT: Client<HttpsConnector<HttpConnector>, Body> = {
    let https = HttpsConnector::new();
    Client::builder().build::<_, Body>(https)
 };
}

pub async fn handler(req: Request<Body>) -> Result<Response<Body>, Box<dyn Error>> {
    let (mut parts, body) = req.into_parts();

    parts.uri = Uri::builder()
        .scheme(TARGET_SCHEME)
        .authority(TARGET_HOST)
        .path_and_query(parts.uri
            .path_and_query()
            .map(|pq| pq.clone())
            .unwrap_or(PathAndQuery::from_static("/"))
        )
        .build()?;
    parts.headers.remove(HOST);
    parts.headers.insert(HOST, TARGET_HOST.parse()?);

    let my_req = Request::from_parts(parts, body);

    info!("Internal request {:?}", my_req);

    let rs = CLIENT.request(my_req)
        .await
        .map_err(|e| Box::new(e) as Box<dyn Error>);

    info!("Internal response {:?}", rs);

    rs
}