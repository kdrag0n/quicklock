use crate::envelope::{RequestEnvelope, AuditStamp};
use crate::error::AppResult;
use crate::{crypto, profile};
use crate::serialize::base64 as serde_b64;
use crate::time::now;

use std::net::SocketAddr;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use base64;
use tracing::log::debug;
use ulid::Ulid;
use std::str;
use axum::{Json, Router};
use axum::extract::{Path, ConnectInfo};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use ring::hmac;
use ring::hmac::HMAC_SHA256;
use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair};
use crate::audit::store::{LogEvent, PairedDevice, STORE};

pub mod store;
pub mod client;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    #[serde(with = "serde_b64")]
    pub client_mac_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub client_id: String,
    #[serde(with = "serde_b64")]
    pub server_pk: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignRequest {
    pub client_id: String,
    #[serde(with = "serde_b64")]
    pub envelope: Vec<u8>,
    #[serde(with = "serde_b64")]
    pub client_mac: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignResponse {
    #[serde(with = "serde_b64")]
    pub stamp: Vec<u8>,
    #[serde(with = "serde_b64")]
    pub server_sig: Vec<u8>,
}

async fn register(req: Json<RegisterRequest>) -> AppResult<impl IntoResponse> {
    debug!("Register: {:?}", req);

    // Client ID = hash of MAC key
    let client_id = base64::encode(crypto::hash(&req.client_mac_key));

    // Generate server keypair for this device
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())?;
    let keypair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref())?;
    STORE.add_device(PairedDevice {
        client_id: client_id.clone(),
        client_mac_key: req.client_mac_key.clone(),
        server_keypair: pkcs8.as_ref().into(),
    });

    Ok(Json(RegisterResponse {
        client_id,
        server_pk: keypair.public_key().as_ref().into(),
    }))
}

async fn sign(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Json<SignRequest>,
) -> AppResult<impl IntoResponse> {
    debug!("Sign: {:?}", req);

    let device = STORE.get_device(&req.client_id)
        .ok_or_else(|| anyhow!("Unknown device"))?;

    // Verify client MAC to make sure this client is authorized
    let mac_key = hmac::Key::new(HMAC_SHA256, &device.client_mac_key);
    hmac::verify(&mac_key, &req.envelope, &req.client_mac)?;

    // Create metadata stamp
    let stamp = AuditStamp {
        envelope_hash: crypto::hash(&req.envelope).into(),
        client_ip: addr.ip().to_string(),
        timestamp: now(),
    };
    let envelope: RequestEnvelope = serde_json::from_slice(&req.envelope)?;

    // Log request
    profile!("audit store", {
        STORE.log_event(&device.client_id, LogEvent {
            id: Ulid::new().into(),
            envelope: envelope.clone(),
            stamp: stamp.clone(),
        });
    });

    // Sign stamp
    let stamp_data = serde_json::to_string(&stamp)?;
    let keypair = Ed25519KeyPair::from_pkcs8(&device.server_keypair)?;
    let sig = profile!("audit sign", {
        keypair.sign(stamp_data.as_bytes())
    });

    // Send our signature to the client for aggregation. Client needs to re-sign
    // with the new envelope that includes metadata.
    Ok(Json(SignResponse {
        stamp: serde_json::to_vec(&stamp)?,
        server_sig: sig.as_ref().into(),
    }))
}

// TODO: auth with replay protection?
async fn get_logs(Path(client_pk): Path<String>) -> AppResult<impl IntoResponse> {
    debug!("Get logs: {}", client_pk);
    Ok(Json(STORE.get_logs(&client_pk)))
}

pub fn service() -> Router {
    Router::new()
        .route("/api/register", post(register))
        .route("/api/sign", post(sign))
        .route("/api/device/:client_pk/logs", get(get_logs))
}
