use std::{collections::HashMap, io::Result, sync::Arc};

use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer,
    dev::Server,
    http::StatusCode,
    web::{self, Bytes},
};

use crate::{
    http::{HttpMethod, Request, utils::decode_query},
    routing::dispatcher::Dispatcher,
};

struct AppState {
    router: Arc<Dispatcher>,
    basepath: String,
}

pub struct Building;
pub struct Ready;

pub struct HuIOServer<S = Building> {
    _state: std::marker::PhantomData<S>,
    server: Option<Server>,
    hostname: String,
    port: u16,
    basepath: String,
    router: Option<Dispatcher>,
}

impl Default for HuIOServer<Building> {
    fn default() -> Self {
        Self::new()
    }
}

impl HuIOServer<Building> {
    pub fn new() -> Self {
        Self {
            _state: std::marker::PhantomData,
            hostname: String::from("0.0.0.0"),
            port: 80,
            basepath: String::from("/"),
            server: None,
            router: None,
        }
    }

    #[must_use]
    pub fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = hostname.to_string();
        self
    }

    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    #[must_use]
    pub fn basepath(mut self, basepath: &str) -> Self {
        let basepath = basepath.trim_end_matches('/').to_string();
        self.basepath = if basepath.is_empty() {
            "/".to_string()
        } else {
            basepath
        };
        self
    }

    #[must_use]
    pub fn router(mut self, router: Dispatcher) -> Self {
        self.router = Some(router);
        self
    }

    pub fn build(mut self) -> Result<HuIOServer<Ready>> {
        let bind_address = format!("{}:{}", &self.hostname, &self.port);
        let basepath = self.basepath.clone();
        let router = Arc::new(self.router.take().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Router not set. Call .router() before .build()",
            )
        })?);

        let server = HttpServer::new(move || {
            let state = web::Data::new(AppState {
                router: router.clone(),
                basepath: basepath.clone(),
            });

            App::new()
                .app_data(state)
                .app_data(web::PayloadConfig::default().limit(1_048_576))
                .default_service(web::route().to(handle_request))
        })
        .bind(&bind_address)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to bind on {}: {}", bind_address, e),
            )
        })?
        .run();

        Ok(HuIOServer {
            _state: std::marker::PhantomData,
            server: Some(server),
            hostname: self.hostname,
            port: self.port,
            basepath: self.basepath,
            router: None,
        })
    }
}

impl HuIOServer<Ready> {
    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn basepath(&self) -> &str {
        &self.basepath
    }

    pub async fn run(self) -> Result<()> {
        match self.server {
            Some(srv) => srv.await,
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Internal error: server instance missing in Ready state",
            )),
        }
    }
}

async fn handle_request(req: HttpRequest, body: Bytes, state: web::Data<AppState>) -> HttpResponse {
    let raw_path = req.path();

    let path = if state.basepath != "/" {
        let prefix_with_slash = format!("{}/", state.basepath);
        if raw_path.starts_with(&prefix_with_slash) {
            raw_path[state.basepath.len()..].to_string()
        } else if raw_path == state.basepath {
            "/".to_string()
        } else {
            return HttpResponse::NotFound().body("Not Found");
        }
    } else {
        raw_path.to_string()
    };

    let method = match HttpMethod::from(req.method().as_str()) {
        Some(m) => m,
        None => return HttpResponse::MethodNotAllowed().finish(),
    };

    let headers = req
        .headers()
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
        .collect();

    let query = decode_query(req.query_string());

    let our_req = Request {
        method,
        path,
        headers,
        body: body.to_vec(),
        params: HashMap::new(),
        query,
    };

    let our_res = state.router.handle(our_req).await;
    let (status, headers, body) = our_res.into_parts();

    let mut builder = HttpResponse::build(
        StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
    );

    for (key, value) in &headers {
        builder.insert_header((key.as_str(), value.as_str()));
    }

    builder.body(body)
}
