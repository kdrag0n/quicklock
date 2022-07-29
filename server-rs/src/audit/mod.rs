use std::net::SocketAddr;
use crate::error::AppResult;
use crate::serialize::base64 as serde_b64;

use std::time::SystemTime;
use anyhow::anyhow;
use bls_signatures::{PrivateKey, PublicKey, Serialize as BlsSerialize, Signature};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use base64;
use ulid::Ulid;
use std::str;
use std::str::FromStr;
use axum::{Json, middleware, Router};
use axum::response::IntoResponse;
use axum::routing::post;
use tower_http::trace::TraceLayer;
use crate::log;
use crate::log::print_request_response;
use crate::audit::store::{AuthEvent, PairedDevice, STORE};
use crate::bls::{sign_aug, verify_aug};

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
    STORE.add_device(PairedDevice {
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

    let device = STORE.get_device(&req.client_pk)
        .ok_or(anyhow!("Unknown device"))?;

    // Verify client signature to make sure this client is authorized
    let client_pk_data = base64::decode(&device.client_pk)?;
    let client_pk = PublicKey::from_bytes(&client_pk_data)?;
    let client_sig = Signature::from_bytes(&req.client_sig)?;
    println!("aug signature: {:?}", base64::encode(&req.client_sig));
    if !verify_aug(&client_sig, &req.message, &[client_pk]) {
        return Err(anyhow!("Client signature invalid").into());
    }

    // Log request
    STORE.log_event(&device.client_pk, AuthEvent {
        id: Ulid::new().into(),
        timestamp: SystemTime::now(),
        payload: req.message.clone(),
    });

    // Sign payload
    let sk = PrivateKey::from_bytes(&device.server_sk)?;
    println!("sign for: {}", std::str::from_utf8(&req.message)?);
    let sig = sign_aug(&sk, &req.message);
    // Aggregate signature
    println!("client pk = {}", base64::encode(client_pk.as_bytes()));
    println!("server pk = {}", base64::encode(sk.public_key().as_bytes()));
    let agg_sig = bls_signatures::aggregate(&[client_sig, sig])?;
    println!("Agg sig dbg: {:?}", agg_sig);
    println!("pk client dbg: {:?}", client_pk);
    println!("pk server dbg: {:?}", sk.public_key());

    // self verify
    println!("self verify1 c = {}",
             bls_signatures::verify_messages(&client_sig, &[req.message.as_slice()], &[client_pk]));
    println!("self verify1 s = {}",
                bls_signatures::verify_messages(&sig, &[req.message.as_slice()], &[sk.public_key()]));
    println!("self verify = {}",
             verify_aug(&agg_sig, &req.message, &[client_pk, sk.public_key()]));

    //
    println!("Agg sig dat = {}", base64::encode(agg_sig.as_bytes()));
    Ok(Json(SignResponse {
        aggregate_sig: agg_sig.as_bytes(),
    }))
}

pub fn service() -> Router {
    Router::new()
        .route("/api/register", post(register))
        .route("/api/sign", post(sign))
}
