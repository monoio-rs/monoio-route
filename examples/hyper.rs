#![allow(clippy::type_complexity)]

use std::{collections::HashMap, net::SocketAddr, pin::Pin, rc::Rc};

use bytes::Bytes;
use http_body_util::Full;
use hyper::{
    body::Incoming,
    server::conn::http1,
    service::{service_fn, Service},
    Method, Request, Response, StatusCode,
};
use monoio::{io::IntoPollIo, net::TcpListener};
use std::future::Future;

type Router<S> = HashMap<Method, monoio_route::Tree<S>>;
type BoxFuture<O> = Pin<Box<dyn Future<Output = O>>>;
type BoxHyperService<Req, Resp, E> = Box<
    dyn Service<Req, Response = Resp, Error = E, Future = BoxFuture<Result<Resp, E>>> + 'static,
>;

async fn serve_http<S, A>(addr: A, service: S) -> std::io::Result<()>
where
    S: Clone
        + Service<Request<Incoming>, Error = hyper::Error, Response = Response<Full<Bytes>>>
        + 'static,
    A: Into<SocketAddr>,
{
    let listener = TcpListener::bind(addr.into())?;
    loop {
        let (stream, _) = listener.accept().await?;
        let stream_poll = monoio_compat::hyper::MonoioIo::new(stream.into_poll_io()?);
        let cloned_svc = service.clone();
        monoio::spawn(async move {
            // Handle the connection from the client using HTTP1 and pass any
            // HTTP requests received on that connection to the `hello` function
            if let Err(err) = http1::Builder::new()
                .timer(monoio_compat::hyper::MonoioTimer)
                .serve_connection(stream_poll, cloned_svc)
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

struct ServiceWrapper<S>(S);
impl<S, Req> Service<Req> for ServiceWrapper<S>
where
    S: Service<Req>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<Result<S::Response, S::Error>>;

    fn call(&self, req: Req) -> Self::Future {
        Box::pin(self.0.call(req))
    }
}

impl<S> ServiceWrapper<S> {
    fn boxed<Req>(
        self,
    ) -> Box<
        dyn Service<
            Req,
            Response = S::Response,
            Error = S::Error,
            Future = BoxFuture<Result<S::Response, S::Error>>,
        >,
    >
    where
        S: Service<Req> + 'static,
        S::Future: 'static,
    {
        Box::new(self)
    }
}

#[derive(Clone)]
struct HyperSvc {
    router: Rc<Router<BoxHyperService<Request<Incoming>, Response<Full<Bytes>>, hyper::Error>>>,
}
impl HyperSvc {
    fn new(
        router: Router<BoxHyperService<Request<Incoming>, Response<Full<Bytes>>, hyper::Error>>,
    ) -> Self {
        Self {
            router: Rc::new(router),
        }
    }
}
impl Service<Request<Incoming>> for HyperSvc {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = BoxFuture<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        // find the subrouter for this request method
        let router = match self.router.get(req.method()) {
            Some(router) => router,
            // if there are no routes for this method, respond with 405 Method Not Allowed
            None => {
                return Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Full::new(Bytes::new()))
                        .unwrap())
                });
            }
        };

        // find the service for this request path
        let svc = match router.at(req.uri().path().as_bytes()) {
            Some((svc, _)) => svc,
            // if we there is no matching service, respond with 404 Not Found
            None => {
                return Box::pin(async move {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Full::new(Bytes::new()))
                        .unwrap())
                });
            }
        };

        // call inner service
        Box::pin(svc.call(req))
    }
}

// GET /
async fn index(_req: Request<Incoming>) -> hyper::Result<Response<Full<Bytes>>> {
    Ok(Response::new(Full::new(Bytes::from("Hello, world!"))))
}

// GET /blog
async fn blog(_req: Request<Incoming>) -> hyper::Result<Response<Full<Bytes>>> {
    Ok(Response::new(Full::new(Bytes::from("..."))))
}

#[monoio::main(threads = 2, timer_enabled = true)]
async fn main() {
    println!("Running http server on 127.0.0.1:3000");
    let mut router = Router::default();
    // GET / => `index`
    router
        .entry(Method::GET)
        .or_default()
        .insert(b"/", ServiceWrapper(service_fn(index)).boxed())
        .unwrap();
    // GET /blog => `blog`
    router
        .entry(Method::GET)
        .or_default()
        .insert(b"/aa/bb/cc", ServiceWrapper(service_fn(blog)).boxed())
        .unwrap();
    router
        .entry(Method::GET)
        .or_default()
        .insert(b"/a/{name}", ServiceWrapper(service_fn(blog)).boxed())
        .unwrap();
    router
        .entry(Method::GET)
        .or_default()
        .insert(b"/a/{*any}", ServiceWrapper(service_fn(blog)).boxed())
        .unwrap();

    let _ = serve_http(([127, 0, 0, 1], 3000), HyperSvc::new(router)).await;
    println!("Http server stopped");
}
