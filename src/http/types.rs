use std::{future::Future, pin::Pin, sync::Arc};

use crate::http::{request::Request, response::Response};

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
pub type Handler = Arc<dyn Fn(Request) -> BoxFuture<Response> + Send + Sync>;
pub type Next = Arc<dyn Fn(Request) -> BoxFuture<Response> + Send + Sync>;
pub type Middleware = Arc<dyn Fn(Request, Next) -> BoxFuture<Response> + Send + Sync>;
