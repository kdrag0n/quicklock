use std::net::SocketAddr;
use std::str::FromStr;

use axum::{Json, middleware, Router};
use axum::response::IntoResponse;
use axum::routing::get;
use tower_http::trace::TraceLayer;
use qlock::log;
use qlock::log::print_request_response;

mod config;
mod attestation;
mod certificates;
mod store;
mod pairing;
mod crypto;
mod actions;
mod homeassistant;
mod request;

use config::CONFIG;

async fn get_entities() -> impl IntoResponse {
    Json(CONFIG.entities.values().collect::<Vec<_>>())
}

pub fn service() -> Router {
    Router::new()
        .route("/api/entity", get(get_entities))
        .nest("/api/pair", pairing::service())
        .nest("/api/unlock", actions::service())
}

#[tokio::main]
async fn main() {
    log::init();

    let app = Router::new()
        .merge(service())
        .merge(qlock::audit::service())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(print_request_response));

    let addr = SocketAddr::from_str("0.0.0.0:3002").unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
