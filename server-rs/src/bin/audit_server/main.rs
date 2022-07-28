use qlock::error::Error;
use qlock::serialize::base64 as serde_b64;

use std::time::SystemTime;
use actix_web::{App, HttpServer, post, Responder, web};
use actix_web::middleware::Logger;
use actix_web::web::Json;
use anyhow::anyhow;
use bls_signatures::{PrivateKey, PublicKey, Serialize as BlsSerialize, Signature};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use base64;
use ulid::Ulid;
use std::str;
use crate::store::{AuthEvent, DataStore, PairedDevice};

mod store;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    client_pk: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterResponse {
    #[serde(with="serde_b64")]
    server_pk: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignRequest {
    client_pk: String,
    #[serde(with="serde_b64")]
    message: Vec<u8>,
    #[serde(with="serde_b64")]
    client_sig: Vec<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SignResponse {
    #[serde(with="serde_b64")]
    aggregate_sig: Vec<u8>,
}

#[post("register")]
async fn register(req: Json<RegisterRequest>) -> Result<impl Responder, Error> {
    println!("Register: {:?}", req);

    // Generate server private key for this device
    let sk = PrivateKey::generate(&mut OsRng);
    DataStore::get().add_device(PairedDevice {
        client_pk: req.client_pk.clone(),
        server_sk: sk.as_bytes(),
    });

    // Return our public key
    Ok(Json(RegisterResponse {
        server_pk: sk.public_key().as_bytes(),
    }))
}

#[post("sign")]
async fn sign(req: Json<SignRequest>) -> Result<impl Responder, Error> {
    println!("Sign: {:?}", req);

    let device = DataStore::get().get_device(&req.client_pk)
        .ok_or(anyhow!("Unknown device"))?;

    // Verify client signature to make sure this client is authorized
    let client_pk_data = base64::decode(&device.client_pk)?;
    let client_pk = PublicKey::from_bytes(&client_pk_data)?;
    let client_sig = Signature::from_bytes(&req.client_sig)?;
    if !client_pk.verify(client_sig, &req.message) {
        return Err(anyhow!("Client signature invalid").into());
    }

    // Log request
    DataStore::get().log_event(&device.client_pk, AuthEvent {
        id: Ulid::new().into(),
        timestamp: SystemTime::now(),
        payload: req.message.clone(),
    });

    // Sign payload
    let sk = PrivateKey::from_bytes(&device.server_sk)?;
    let sig = sk.sign(&req.message);
    // Aggregate signature
    let agg_sig = bls_signatures::aggregate(&[sig, client_sig])?;

    Ok(Json(SignResponse {
        aggregate_sig: agg_sig.as_bytes(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .service(register)
                    .service(sign),
            )
    })
    .bind(("0.0.0.0", 9001))?
    .run()
    .await
}
