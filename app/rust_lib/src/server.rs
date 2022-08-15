use std::net::SocketAddr;
use std::str::FromStr;
use std::{env, fs, thread};

use axum::{middleware, Router};
use log::info;
use tower_http::trace::TraceLayer;
use qlock::log::print_request_response;

const CONFIG_DATA: &str = r#"{
  "haApiKey": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJlYWRiZGFmZDY1Zjc0MjZjYWZhNzc3ZmNhMTdiNjY1MiIsImlhdCI6MTY1NjcwODcyMSwiZXhwIjoxOTcyMDY4NzIxfQ.wYKpYl9dzPKP-0VH8UlbLFQWFlYlBIWq03RYJc_vUQE",
  "entities": {
    "front": {
      "id": "front",
      "name": "Front Door",
      "haEntity": "lock.assure_touchscren_deadbolt"
    },
    "back": {
      "id": "back",
      "name": "Back Door",
      "haEntity": "lock.assure_touchscren_deadbolt"
    }
  }
}"#;

#[tokio::main]
async fn server_thread() {
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

pub fn start_bg() {
    // log::init();

    info!("Spawn thread");
    thread::spawn(|| {
        let _ = fs::create_dir("/data/user/0/dev.kdrag0n.quicklock/files");
        env::set_current_dir("/data/user/0/dev.kdrag0n.quicklock/files").unwrap();
        fs::write("config.json", CONFIG_DATA).unwrap();
        server_thread()
    });
}
