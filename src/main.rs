use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use hyper::{Request, Body, Response, Server};
use proxy_handler::handler;
use leaky_bucket::LeakyBucket;
use log::{info, error};
use lazy_static::lazy_static;

mod proxy_handler;

lazy_static! {
 static ref RATE_LIMITER: LeakyBucket = LeakyBucket::builder()
    .max(5)
    .tokens(5)
    .build()
    .expect("Unable to construct rate limiter");
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = "0.0.0.0:3000".parse().expect("Bad bind address");

    let make_svc = make_service_fn(|_| {
        async {
            Ok::<_, Infallible>(service_fn(|req: Request<Body>| async {
                Ok::<_, Infallible>({
                    info!("Pending External request {:?}", req);

                    RATE_LIMITER.acquire(10).await.expect("Rate limiter error");

                    info!("Performing External request {:?}", req);

                    let result = handler(req).await;
                    match result {
                        Ok(response) => {
                            info!("External response {:?}", response);
                            response
                        }
                        Err(e) => {
                            error!("Proxy error: {}", e);
                            let rs = Response::builder()
                                .status(500)
                                .body(Body::empty())
                                .expect("Error constructing error response");

                            info!("External response {:?}", rs);

                            rs
                        }
                    }
                })
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    let graceful = server.with_graceful_shutdown((|| async {
        tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
    })());

    if let Err(e) = graceful.await {
        error!("Server error: {}", e);
    }
}