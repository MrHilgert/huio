use std::{collections::HashMap, future::Future, sync::Arc};

use crate::http::{BoxFuture, Handler, HttpMethod, Middleware, Request, Response};

macro_rules! impl_method {
    ($name:ident, $method:expr) => {
        #[must_use]
        pub fn $name<F, Fut>(self, f: F) -> Self
        where
            F: Fn(Request) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Response> + Send + 'static,
        {
            self.add_handler($method, f)
        }
    };
}

pub struct Router {
    pub(crate) path: String,
    pub(crate) handlers: HashMap<HttpMethod, Handler>,
    pub(crate) middlewares: Vec<Middleware>,
    pub(crate) children: Vec<Router>,
}

impl Router {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            handlers: HashMap::new(),
            middlewares: Vec::new(),
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn middleware(mut self, m: Middleware) -> Self {
        self.middlewares.push(m);
        self
    }

    #[must_use]
    pub fn nest(mut self, child: Router) -> Self {
        self.children.push(child);
        self
    }

    fn add_handler<F, Fut>(mut self, method: HttpMethod, f: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        self.handlers.insert(
            method,
            Arc::new(move |req| Box::pin(f(req)) as BoxFuture<Response>),
        );
        self
    }

    impl_method!(get, HttpMethod::GET);
    impl_method!(post, HttpMethod::POST);
    impl_method!(put, HttpMethod::PUT);
    impl_method!(delete, HttpMethod::DELETE);
    impl_method!(patch, HttpMethod::PATCH);
    impl_method!(head, HttpMethod::HEAD);
    impl_method!(options, HttpMethod::OPTIONS);
}
