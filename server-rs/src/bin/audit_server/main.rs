use std::net::SocketAddr;
use qlock::error::{AppResult, Error};
use qlock::serialize::base64 as serde_b64;

use std::time::SystemTime;
use anyhow::anyhow;
use bls_signatures::{PrivateKey, PublicKey, Serialize as BlsSerialize, Signature};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use base64;
use ulid::Ulid;
use std::str;
use std::str::FromStr;
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::post;
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

async fn register(req: Json<RegisterRequest>) -> AppResult<impl IntoResponse> {
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

async fn sign(req: Json<SignRequest>) -> AppResult<impl IntoResponse> {
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

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let app = Router::new()
        .route("/api/register", post(register))
        .route("/api/sign", post(sign));

    let addr = SocketAddr::from_str("0.0.0.0:9001").unwrap();
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
