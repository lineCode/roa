use super::method_not_allowed;
use doc_comment::doc_comment;
use roa_core::http::Method;
use roa_core::{async_trait, Context, Endpoint, Result};
use std::collections::HashMap;

macro_rules! impl_http_methods {
    ($end:ident, $method:expr) => {
        pub fn $end(mut self, endpoint: impl for<'a> Endpoint<'a, S>) -> Self {
            self.0.insert($method, Box::new(endpoint));
            self
        }
    };
}

macro_rules! impl_http_functions {
    ($end:ident, $method:expr) => {
        doc_comment! {
        concat!("Function to construct dispatcher with ", stringify!($method), " and an endpoint.

You can use it as follow:

```
use roa_core::{App, Context, Result};
use roa_router::", stringify!($end), ";

async fn end(ctx: &mut Context<()>) -> Result {
    Ok(())
}

let app = App::new(()).end(", stringify!($end), "(end));
```"),
                pub fn $end<S>(endpoint: impl for<'a> Endpoint<'a, S>) -> Dispatcher<S> {
                        Dispatcher::<S>::default().$end(endpoint)
                    }
                }
    };
}

/// An endpoint to dispatch requests by http method.
pub struct Dispatcher<S>(HashMap<Method, Box<dyn for<'a> Endpoint<'a, S>>>);

impl_http_functions!(get, Method::GET);
impl_http_functions!(post, Method::POST);
impl_http_functions!(put, Method::PUT);
impl_http_functions!(patch, Method::PATCH);
impl_http_functions!(options, Method::OPTIONS);
impl_http_functions!(delete, Method::DELETE);
impl_http_functions!(head, Method::HEAD);
impl_http_functions!(trace, Method::TRACE);
impl_http_functions!(connect, Method::CONNECT);

impl<S> Dispatcher<S> {
    impl_http_methods!(get, Method::GET);
    impl_http_methods!(post, Method::POST);
    impl_http_methods!(put, Method::PUT);
    impl_http_methods!(patch, Method::PATCH);
    impl_http_methods!(options, Method::OPTIONS);
    impl_http_methods!(delete, Method::DELETE);
    impl_http_methods!(head, Method::HEAD);
    impl_http_methods!(trace, Method::TRACE);
    impl_http_methods!(connect, Method::CONNECT);
}

impl<S> Default for Dispatcher<S> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

#[async_trait(?Send)]
impl<'a, S> Endpoint<'a, S> for Dispatcher<S>
where
    S: 'static,
{
    #[inline]
    async fn call(&'a self, ctx: &'a mut Context<S>) -> Result<()> {
        match self.0.get(ctx.method()) {
            Some(endpoint) => endpoint.call(ctx).await,
            None => method_not_allowed(ctx.method()),
        }
    }
}
