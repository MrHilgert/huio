# huio

HTTP framework for Rust built on top of [Actix-Web](https://actix.rs/).

## Installation

```toml
[dependencies]
huio = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick start

```rust
use huio::{HuIOServer, http::{Request, Response}, routing::{Dispatcher, Router}};

async fn hello(_req: Request) -> Response {
    Response::ok("Hello, world!")
}

#[tokio::main]
async fn main() {
    let router = Router::new("/").get(hello);

    HuIOServer::default()
        .hostname("0.0.0.0")
        .port(8080)
        .router(Dispatcher::new(router))
        .build()
        .unwrap()
        .run()
        .await
        .unwrap();
}
```

## Routing

```rust
use huio::routing::Router;

let api = Router::new("/users")
    .get(list_users)
    .post(create_user)
    .nest(
        Router::new("/:id")
            .get(get_user)
            .put(update_user)
            .delete(delete_user),
    );

let root = Router::new("/").nest(api);
```

Path parameters use the `:name` syntax and are accessed via `req.param("name")`. Trailing slashes are normalized automatically. `HEAD` requests are handled automatically if a `GET` handler is registered.

## Request

```rust
req.path()           // &str
req.method           // HttpMethod
req.param("id")      // Option<&str>  — path parameter
req.query("page")    // Option<&str>  — query string parameter
req.query_all()      // &HashMap<String, String>
req.headers          // HashMap<String, String>
req.body             // Vec<u8>
```

## Response

```rust
Response::ok("text")               // 200 text/plain
Response::json(value)              // 200 application/json (accepts any impl Serialize)
Response::not_found()              // 404
Response::method_not_allowed(&[])  // 405
Response::internal_error()         // 500

// Builder methods (chainable):
Response::ok("text")
    .status(201).unwrap()
    .header("X-Custom", "value")
```

## Middleware

`Middleware` is `Arc<dyn Fn(Request, Next) -> BoxFuture<Response> + Send + Sync>`.  
Middlewares are applied per-router and inherited by nested routers.

```rust
use std::sync::Arc;
use huio::http::Middleware;

let logger: Middleware = Arc::new(|req, next| {
    Box::pin(async move {
        println!("→ {:?} {}", req.method, req.path());
        let start = std::time::Instant::now();
        let res = next(req).await;
        println!("← {} {:?}", res.status_code(), start.elapsed());
        res
    })
});

let router = Router::new("/api")
    .middleware(logger)
    .get(handler);
```

## Server configuration

```rust
HuIOServer::default()
    .hostname("127.0.0.1")   // default: "0.0.0.0"
    .port(3000)               // default: 80
    .basepath("/v1")          // optional base prefix
    .router(Dispatcher::new(root))
    .build()?
    .run()
    .await?;
```

`build()` returns `HuIOServer<Ready>`, which exposes `.hostname()`, `.port()`, `.basepath()` getters and `.run()`.

## HTTP methods

`GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `HEAD`, `OPTIONS` — registered via same-named builder methods on `Router`.

## License

MIT