use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::get;

pub mod model;
pub mod config;
mod attestation;
mod certificates;
mod store;
mod pairing;
mod crypto;
pub mod actions;
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
