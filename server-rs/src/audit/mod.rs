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
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use crate::audit::store::{LogEvent, PairedDevice, STORE};
use crate::bls::{aggregate_pks_multi, aggregate_sigs_multi, sign_aug, verify_aug};

pub mod store;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub client_pk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    #[serde(with="serde_b64")]
    pub aggregate_pk: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignRequest {
    pub client_pk: String,
    #[serde(with="serde_b64")]
    pub enc_message: Vec<u8>,
    #[serde(with="serde_b64")]
    pub enc_nonce: Vec<u8>,
    #[serde(with="serde_b64")]
    pub message_hash: Vec<u8>,
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
    let client_pk_data = base64::decode(&req.client_pk)?;
    let client_pk = PublicKey::from_bytes(&client_pk_data)?;
    let sk = PrivateKey::generate(&mut OsRng);
    STORE.add_device(PairedDevice {
        client_pk: req.client_pk.clone(),
        server_sk: sk.as_bytes(),
    });

    // Return aggregated public key
    let server_pk = sk.public_key();
    let agg_pk = aggregate_pks_multi(&[&client_pk, &server_pk]);
    Ok(Json(RegisterResponse {
        aggregate_pk: agg_pk.as_bytes(),
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
    if !client_pk.verify(client_sig, &req.message_hash) {
        return Err(anyhow!("Client signature invalid").into());
    }

    // Log request
    STORE.log_event(&device.client_pk, LogEvent {
        id: Ulid::new().into(),
        timestamp: SystemTime::now(),
        enc_message: req.enc_message.clone(),
        enc_nonce: req.enc_nonce.clone(),
    });

    // Sign payload
    let sk = PrivateKey::from_bytes(&device.server_sk)?;
    let sig = sk.sign(&req.message_hash);
    // Aggregate signature
    let agg_sig = aggregate_sigs_multi(&[
        (&client_sig, &client_pk),
        (&sig, &sk.public_key()),
    ]);

    Ok(Json(SignResponse {
        aggregate_sig: agg_sig.as_bytes(),
    }))
}

// TODO: auth with replay protection?
async fn get_logs(Path(client_pk): Path<String>) -> AppResult<impl IntoResponse> {
    println!("Get logs: {}", client_pk);
    Ok(Json(STORE.get_logs(&client_pk)))
}

pub fn service() -> Router {
    Router::new()
        .route("/api/register", post(register))
        .route("/api/sign", post(sign))
        .route("/api/device/:client_pk/logs", get(get_logs))
}
