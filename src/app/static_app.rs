use crate::{Context, DynMiddleware, Model, Next, _next};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server};
use std::convert::Infallible;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct StaticApp<M: Model = ()> {
    handler: Box<dyn DynMiddleware<M>>,
}

impl<M: Model> StaticApp<M> {
    pub fn new() -> Self {
        Self {
            handler: Box::new(|ctx, next| next(ctx)),
        }
    }

    // TODO: replace DynMiddleware with Middleware
    pub fn register(self, middleware: impl DynMiddleware<M>) -> Self {
        let middleware = Arc::new(middleware);
        Self {
            handler: Box::new(move |ctx, next| {
                let middleware_ref = middleware.clone();
                let current: Next<M> = Box::new(move |ctx| middleware_ref(ctx, next));
                (self.handler)(ctx, current)
            }),
        }
    }

    async fn handle(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let mut context = Context::new(req, self);
        (self.handler)(&mut context, Box::new(_next)).await?;
        Ok(Response::new(Body::empty()))
    }

    pub fn leak(self) -> &'static Self {
        Box::leak(Box::new(self))
    }

    pub fn listen(
        &'static self,
        addr: &SocketAddr,
    ) -> impl 'static + Future<Output = Result<(), Error>> {
        let make_svc = make_service_fn(move |_conn| {
            async move { Ok::<_, Infallible>(service_fn(move |req| self.handle(req))) }
        });
        Server::bind(addr).serve(make_svc)
    }
}

#[cfg(test)]
mod tests {
    use super::StaticApp;
    #[test]
    fn test() {
        let x = 1;
        let a: Box<dyn Fn() -> i32> = Box::new(|| x);
    }
}
