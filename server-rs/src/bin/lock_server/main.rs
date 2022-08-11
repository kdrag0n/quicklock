use std::net::SocketAddr;
use std::str::FromStr;

use axum::{middleware, Router};
use tower_http::trace::TraceLayer;
use qlock::log;
use qlock::log::print_request_response;

#[tokio::main]
async fn main() {
    log::init();

    let app = Router::new()
        .merge(qlock::lock::service())
        .merge(qlock::audit::service())
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(print_request_response));

    let addr = SocketAddr::from_str("0.0.0.0:3002").unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
