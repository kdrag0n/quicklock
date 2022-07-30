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
use axum::{Json, Router};
use axum::response::IntoResponse;
use axum::routing::post;
use crate::audit::store::{AuthEvent, PairedDevice, STORE};
use crate::bls::{sign_aug, verify_aug};

mod store;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub client_pk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    #[serde(with="serde_b64")]
    pub server_pk: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignRequest {
    pub client_pk: String,
    #[serde(with="serde_b64")]
    pub message: Vec<u8>,
    #[serde(with="serde_b64")]
    pub client_sig: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignResponse {
    #[serde(with="serde_b64")]
    pub aggregate_sig: Vec<u8>,
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
    let sig = sign_aug(&sk, &req.message);
    // Aggregate signature
    let agg_sig = bls_signatures::aggregate(&[client_sig, sig])?;

    Ok(Json(SignResponse {
        aggregate_sig: agg_sig.as_bytes(),
    }))
}

pub fn service() -> Router {
    Router::new()
        .route("/api/register", post(register))
        .route("/api/sign", post(sign))
}