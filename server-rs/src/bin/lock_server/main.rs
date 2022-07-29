use std::net::SocketAddr;
use std::str::FromStr;

use axum::{Json, middleware, Router};
use axum::body::{Body, Bytes};
use axum::http::{Request, Response, StatusCode};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::get;
use tower_http::trace::TraceLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use config::CONFIG;

mod config;
mod attestation;
mod certificates;
mod store;
mod pairing;
mod crypto;
mod actions;
mod homeassistant;

async fn get_entities() -> impl IntoResponse {
    Json(CONFIG.entities.values().collect::<Vec<_>>())
}

#[tokio::main]
async fn main() {
    // pretty_env_logger::init();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "lock_server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/api/entity", get(get_entities))
        .nest("/api/pair", pairing::service())
        .nest("/api/unlock", actions::service())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(print_request_response));

    let addr = SocketAddr::from_str("0.0.0.0:3002").unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn print_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
    where
        B: axum::body::HttpBody<Data = Bytes>,
        B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} body = {:?}", direction, body);
    }

    Ok(bytes)
}
