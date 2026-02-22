use std::{collections::HashMap, sync::Arc};

use crate::{
    http::{Handler, HttpMethod, Middleware, Next, Request, Response, utils::decode_component},
    routing::Router,
};

fn normalize_path(path: &str) -> &str {
    if path.len() > 1 && path.ends_with('/') {
        &path[..path.len() - 1]
    } else {
        path
    }
}

enum ResolveResult<'a> {
    Found(&'a Handler, HashMap<String, String>, Vec<Middleware>),
    MethodNotAllowed(Vec<String>),
    NotFound,
}

pub struct Dispatcher {
    root: Router,
}

impl Dispatcher {
    pub fn new(root: Router) -> Self {
        Self { root }
    }

    pub async fn handle(&self, req: Request) -> Response {
        let normalized_path = normalize_path(&req.path).to_string();
        let req = Request {
            path: normalized_path,
            ..req
        };

        match self.resolve(&self.root, &req, "", &[]) {
            ResolveResult::Found(handler, params, middlewares) => {
                let is_head = req.method == HttpMethod::HEAD;
                let mut req = req;
                req.params = params;
                let mut res = Self::call_with_middlewares(handler, middlewares, req).await;
                if is_head {
                    res.clear_body();
                }
                res
            }
            ResolveResult::MethodNotAllowed(allowed) => {
                let refs: Vec<&str> = allowed.iter().map(|s| s.as_str()).collect();
                Response::method_not_allowed(&refs)
            }
            ResolveResult::NotFound => Response::not_found(),
        }
    }

    fn resolve<'a>(
        &'a self,
        route: &'a Router,
        req: &Request,
        prefix: &str,
        inherited_middlewares: &[Middleware],
    ) -> ResolveResult<'a> {
        let full_path = format!("{}{}", prefix, route.path);
        let mut params = HashMap::new();

        if self.match_path(&full_path, &req.path, &mut params) {
            let method = if req.method == HttpMethod::HEAD
                && !route.handlers.contains_key(&HttpMethod::HEAD)
            {
                &HttpMethod::GET
            } else {
                &req.method
            };

            if let Some(handler) = route.handlers.get(method) {
                let middlewares = inherited_middlewares
                    .iter()
                    .chain(route.middlewares.iter())
                    .cloned()
                    .collect();
                return ResolveResult::Found(handler, params, middlewares);
            } else if !route.handlers.is_empty() {
                let mut allowed: Vec<String> =
                    route.handlers.keys().map(|m| m.to_string()).collect();
                if route.handlers.contains_key(&HttpMethod::GET)
                    && !route.handlers.contains_key(&HttpMethod::HEAD)
                {
                    allowed.push(HttpMethod::HEAD.to_string());
                }
                allowed.sort();
                return ResolveResult::MethodNotAllowed(allowed);
            }
        }

        if !route.children.is_empty() {
            let next_middlewares: Vec<Middleware> = inherited_middlewares
                .iter()
                .chain(route.middlewares.iter())
                .cloned()
                .collect();

            for child in &route.children {
                match self.resolve(child, req, &full_path, &next_middlewares) {
                    ResolveResult::NotFound => continue,
                    other => return other,
                }
            }
        }

        ResolveResult::NotFound
    }

    fn match_path(&self, pattern: &str, path: &str, params: &mut HashMap<String, String>) -> bool {
        let mut p_parts = pattern.split('/');
        let mut r_parts = path.split('/');

        loop {
            match (p_parts.next(), r_parts.next()) {
                (None, None) => return true,
                (Some(p), Some(r)) => {
                    if p.starts_with(':') {
                        params.insert(p[1..].to_string(), decode_component(r));
                    } else if p != r {
                        return false;
                    }
                }
                _ => return false,
            }
        }
    }

    async fn call_with_middlewares(
        handler: &Handler,
        middlewares: Vec<Middleware>,
        req: Request,
    ) -> Response {
        let handler = handler.clone();

        let chain: Next = middlewares.iter().rev().fold(handler, |next, middleware| {
            let middleware = middleware.clone();
            Arc::new(move |req: Request| {
                let next = next.clone();
                let middleware = middleware.clone();
                Box::pin(async move { (middleware)(req, next).await })
            })
        });

        chain(req).await
    }
}
