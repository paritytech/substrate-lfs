use futures::future::{self, Future};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::convert::Infallible;
use std::marker::PhantomData;
use hyper::server::conn::AddrStream;
use hyper::service::{Service, make_service_fn};
use hyper::{http, Body, Request, Response, Server, StatusCode};
// use sc_client::Client;
use sp_lfs_cache::Cache;
use sp_lfs_core::LfsId;

pub struct HelloWorld<C, L>(C, PhantomData<L>);

impl<C, LfsId> Service<Request<Body>> for HelloWorld<C, LfsId>
where
	C: Cache<LfsId>,
	LfsId: sp_lfs_core::LfsId,
{
    type Response = Response<Body>;
    type Error = http::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
		// remove the leading slash
		let (_, pure_path) = req.uri().path().split_at(1);
		println!("asked for {:?}", pure_path);
		if let Ok(base) = base64::decode_config(pure_path, base64::URL_SAFE) {
			if let Ok(id) = LfsId::try_from(base) {
				println!("valid ID found: {:?}", id);
				if let Ok(data) = self.0.get(&id) {
					return future::ok(Response::new(data.into()))
				}
			}
		}
		let resp = Response::builder()
			.status(StatusCode::NOT_FOUND)
			.body(Body::from("404 - Not found"))
			.expect("Building this simple response doesn't fail. qed");
        future::ok(resp)
    }
}

pub struct MakeSvc<C, L>(C, PhantomData<L>);

impl<C, L, T> Service<T> for MakeSvc<C, L>
where
	C: Cache<L> + Clone + Send,
	L: sp_lfs_core::LfsId + Send,
{
    type Response = HelloWorld<C, L>;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _: T) -> Self::Future {
        future::ok(HelloWorld(self.0.clone(), Default::default()))
    }
}



pub async fn start_server<C, LfsId>(cache: C) -> ()
where
	C: Cache<LfsId> + Clone + 'static + Send,
	LfsId: sp_lfs_core::LfsId + 'static,
{
	// This is our socket address...
	let addr = ([127, 0, 0, 1], 8080).into();

	// let service = HelloWorld(cache, Default::default());

	let server = Server::bind(&addr).serve(MakeSvc(cache, Default::default()));
	if let Err(e) = server.await {
        println!("server error: {}", e);
    }
}
