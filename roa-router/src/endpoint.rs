use http::Method;
use roa_core::{Context, DynHandler, Handler, Middleware, Model, Next, Status};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

pub struct Endpoint<M: Model> {
    path: &'static str,
    middleware: Middleware<M>,
    handle_map: HashMap<Method, Arc<DynHandler<M>>>,
}

impl<M: Model> Endpoint<M> {
    pub fn new(path: &'static str) -> Self {
        Self {
            path,
            middleware: Middleware::new(),
            handle_map: HashMap::new(),
        }
    }

    pub fn join<F>(
        &mut self,
        middleware: impl 'static + Sync + Send + Fn(Context<M>, Next) -> F,
    ) -> &mut Self
    where
        F: 'static + Future<Output = Result<(), Status>> + Send,
    {
        self.middleware.join(middleware);
        self
    }

    pub fn gate<F>(
        &mut self,
        methods: &[Method],
        handler: impl 'static + Sync + Send + Fn(Context<M>) -> F,
    ) -> &mut Self
    where
        F: 'static + Send + Future<Output = Result<(), Status>>,
    {
        let dyn_handler: Arc<DynHandler<M>> = Arc::from(Box::new(handler).dynamic());
        for method in methods {
            self.handle_map.insert(method.clone(), dyn_handler.clone());
        }
        self
    }

    pub fn get<F>(&mut self, handler: impl 'static + Sync + Send + Fn(Context<M>) -> F) -> &mut Self
    where
        F: 'static + Send + Future<Output = Result<(), Status>>,
    {
        self.gate([Method::GET].as_ref(), handler)
    }

    pub fn all<F>(&mut self, handler: impl 'static + Sync + Send + Fn(Context<M>) -> F) -> &mut Self
    where
        F: 'static + Send + Future<Output = Result<(), Status>>,
    {
        self.gate(
            [
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::OPTIONS,
                Method::DELETE,
                Method::HEAD,
                Method::TRACE,
                Method::CONNECT,
            ]
            .as_ref(),
            handler,
        )
    }
}