use qlock::error::Error;
use qlock::serialize::base64 as serde_b64;

use std::time::SystemTime;
use actix_web::{get, App, HttpServer, post, Responder, web};
use actix_web::middleware::Logger;
use actix_web::web::Json;
use anyhow::anyhow;
use bls_signatures::{PrivateKey, PublicKey, Serialize as BlsSerialize, Signature};
use rand::rngs::OsRng;
use base64;
use ulid::Ulid;
use std::str;
use config::CONFIG;

mod config;
mod attestation;
mod certificates;
mod store;
mod pairing;
mod crypto;
mod actions;
mod homeassistant;

#[get("entity")]
async fn get_entities() -> Result<impl Responder, Error> {
    Ok(Json(CONFIG.entities.values().collect::<Vec<_>>()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .service(get_entities)
                    .service(pairing::service())
                    .service(actions::service())
            )
    })
        .bind(("0.0.0.0", 3002))?
        .run()
        .await
}
