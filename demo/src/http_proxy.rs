use futures::future::{self, Future};
use std::convert::Infallible;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use sc_client::Client;

async fn hello(req: Request<Body>) -> Result<Response<Body>, Infallible> {
	let body_str = format!("{:?}", req);
    Ok(Response::new("Hello, World".into()))
}

pub async fn start_server() -> () {
	// This is our socket address...
	let addr = ([127, 0, 0, 1], 8080).into();

	// A `Service` is needed for every connection.
	let make_svc = make_service_fn(|socket: &AddrStream| async {
        Ok::<_, Infallible>(service_fn(hello))
	});

	let server = Server::bind(&addr).serve(make_svc);
	if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}
