use crate::{
    default_status_handler, Context, DynHandler, DynStatusHandler, Model, Request, Response,
    Status, StatusHandler, TargetHandler,
};
use futures::task::Poll;
use http::{Request as HttpRequest, Response as HttpResponse};
use hyper::server::conn::{AddrIncoming, AddrStream};
use hyper::service::Service;
use hyper::Body as HyperBody;
use hyper::Server;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

pub struct App<M: Model> {
    handler: Arc<DynHandler<M::State>>,
    status_handler: Arc<DynStatusHandler<M::State>>,
    pub(crate) model: Arc<M>,
}

pub struct HttpService<M: Model> {
    app: App<M>,
    addr: SocketAddr,
}

impl<M: Model> App<M> {
    pub fn new(handler: Arc<DynHandler<M::State>>, model: Arc<M>) -> Self {
        Self {
            handler,
            status_handler: Arc::from(Box::new(default_status_handler).dynamic()),
            model,
        }
    }

    pub fn handle_status<F>(
        &mut self,
        handler: impl StatusHandler<M::State, StatusFuture = F>,
    ) -> &mut Self
    where
        F: 'static + Future<Output = Result<(), Status>> + Send,
    {
        self.status_handler = Arc::from(Box::new(handler).dynamic());
        self
    }

    pub fn handle_status_fn<F>(
        &mut self,
        handler: impl 'static + Sync + Send + Fn(Context<M::State>, Status) -> F,
    ) -> &mut Self
    where
        F: 'static + Future<Output = Result<(), Status>> + Send,
    {
        self.handle_status(handler)
    }

    pub fn listen(&self, addr: SocketAddr) -> hyper::Server<AddrIncoming, App<M>> {
        log::info!("Server is listening on: http://{}", &addr);
        Server::bind(&addr).serve(self.clone())
    }
}

macro_rules! impl_poll_ready {
    () => {
        fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    };
}

type AppFuture<M> =
    Pin<Box<dyn 'static + Future<Output = Result<HttpService<M>, std::io::Error>> + Send>>;

impl<M: Model> Service<&AddrStream> for App<M> {
    type Response = HttpService<M>;
    type Error = std::io::Error;
    type Future = AppFuture<M>;
    impl_poll_ready!();
    fn call(&mut self, stream: &AddrStream) -> Self::Future {
        let addr = stream.remote_addr();
        let app = self.clone();
        Box::pin(async move { Ok(HttpService::new(app, addr)) })
    }
}

type HttpFuture =
    Pin<Box<dyn 'static + Future<Output = Result<HttpResponse<HyperBody>, Status>> + Send>>;

impl<M: Model> Service<HttpRequest<HyperBody>> for HttpService<M> {
    type Response = HttpResponse<HyperBody>;
    type Error = Status;
    type Future = HttpFuture;
    impl_poll_ready!();
    fn call(&mut self, req: HttpRequest<HyperBody>) -> Self::Future {
        let service = self.clone();
        Box::pin(async move { Ok(service.serve(req.into()).await?.into()) })
    }
}

impl<M: Model> HttpService<M> {
    pub fn new(app: App<M>, addr: SocketAddr) -> Self {
        Self { app, addr }
    }

    pub async fn serve(&self, req: Request) -> Result<Response, Status> {
        let mut context = Context::new(req, self.app.clone(), self.addr);
        let app = self.app.clone();
        if let Err(status) = (app.handler)(context.clone()).await {
            (app.status_handler)(context.clone(), status).await?;
        }
        Ok(std::mem::take(&mut context.response))
    }
}

impl<M: Model> Clone for App<M> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            status_handler: self.status_handler.clone(),
            model: self.model.clone(),
        }
    }
}

impl<M: Model> Clone for HttpService<M> {
    fn clone(&self) -> Self {
        Self {
            app: self.app.clone(),
            addr: self.addr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{App, HttpService};
    use crate::Request;
    use std::time::Instant;

    #[tokio::test]
    async fn gate_simple() -> Result<(), Box<dyn std::error::Error>> {
        let app = App::builder()
            .handle_fn(|_ctx, next| {
                async move {
                    let inbound = Instant::now();
                    next().await?;
                    println!("time elapsed: {} ms", inbound.elapsed().as_millis());
                    Ok(())
                }
            })
            .model(());
        let _resp = HttpService::new(app, "127.0.0.1:8080".parse()?)
            .serve(Request::new())
            .await?;
        Ok(())
    }
}
