use crate::error::AppResult;
use crate::serialize::base64 as serde_b64;
use crate::time::now;

use std::net::SocketAddr;
use anyhow::anyhow;
use bls_signatures::{PrivateKey, PublicKey, Serialize as BlsSerialize, Signature};
use chacha20poly1305::{aead::{Aead, Nonce}, XChaCha20Poly1305, KeyInit};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use base64;
use ulid::Ulid;
use std::str;
use axum::{Json, Router};
use axum::extract::{Path, ConnectInfo};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use crate::audit::store::{LogEvent, PairedDevice, STORE};
use crate::bls::{aggregate_pks_multi, aggregate_sigs_multi};

pub mod store;
pub mod client;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub client_pk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    #[serde(with = "serde_b64")]
    pub server_pk: Vec<u8>,
    #[serde(with = "serde_b64")]
    pub aggregate_pk: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignRequest {
    pub client_pk: String,
    #[serde(with = "serde_b64")]
    pub envelope: Vec<u8>,
    #[serde(with = "serde_b64")]
    pub client_sig: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignResponse {
    #[serde(with = "serde_b64")]
    pub new_envelope: Vec<u8>, // with metadata injected
    #[serde(with = "serde_b64")]
    pub server_sig: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEnvelope {
    // XChaCha20-Poly1305
    #[serde(with = "serde_b64")]
    pub enc_payload: Vec<u8>,
    #[serde(with = "serde_b64")]
    pub enc_nonce: Vec<u8>, // 192 bits
    // Added by audit server
    pub public_metadata: Option<RequestPublicMetadata>,
}

impl RequestEnvelope {
    pub fn seal<T>(request: &T, enc_key: &[u8]) -> AppResult<RequestEnvelope>
    where
        T: Serialize
    {
        let payload = serde_json::to_string(request)?;

        let cipher = XChaCha20Poly1305::new_from_slice(enc_key)
            .map_err(|_| anyhow!("Bad key"))?;
        let nonce_bytes: [u8; 24] = rand::random();
        let nonce = <Nonce<XChaCha20Poly1305>>::from_slice(&nonce_bytes);
        let enc_payload = cipher.encrypt(nonce, payload.as_bytes())
            .map_err(|_| anyhow!("Failed to encrypt"))?;

        Ok(RequestEnvelope {
            enc_payload,
            enc_nonce: nonce_bytes.into(),
            // Filled by audit server
            public_metadata: None,
        })
    }

    pub fn open<T>(&self, enc_key: &[u8]) -> AppResult<T>
    where
        T: DeserializeOwned
    {
        // Decrypt request contents
        let cipher = XChaCha20Poly1305::new_from_slice(enc_key)
            .map_err(|_| anyhow!("Bad key"))?;
        let nonce = <Nonce<XChaCha20Poly1305>>::from_slice(&self.enc_nonce);
        let payload = cipher.decrypt(nonce, &*self.enc_payload)
            .map_err(|_| anyhow!("Failed to decrypt"))?;

        // Decode payload
        let request = serde_json::from_slice(&payload)?;
        Ok(request)
    }

    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPublicMetadata {
    pub client_ip: String,
    pub timestamp: u64,
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
        server_pk: server_pk.as_bytes(),
        aggregate_pk: agg_pk.as_bytes(),
    }))
}

async fn sign(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Json<SignRequest>,
) -> AppResult<impl IntoResponse> {
    println!("Sign: {:?}", req);

    let device = STORE.get_device(&req.client_pk)
        .ok_or_else(|| anyhow!("Unknown device"))?;

    // Verify client signature to make sure this client is authorized
    let client_pk_data = base64::decode(&device.client_pk)?;
    let client_pk = PublicKey::from_bytes(&client_pk_data)?;
    let client_sig = Signature::from_bytes(&req.client_sig)?;
    if !client_pk.verify(client_sig, &req.envelope) {
        return Err(anyhow!("Client signature invalid").into());
    }

    // Add metadata
    let metadata = RequestPublicMetadata {
        client_ip: addr.ip().to_string(),
        timestamp: now(),
    };
    let mut envelope: RequestEnvelope = serde_json::from_slice(&req.envelope)?;
    envelope = RequestEnvelope {
        public_metadata: Some(metadata),
        ..envelope
    };

    // Log request
    STORE.log_event(&device.client_pk, LogEvent {
        id: Ulid::new().into(),
        envelope: envelope.clone(),
    });

    // Sign payload
    let new_envelope = envelope.serialize();
    let sk = PrivateKey::from_bytes(&device.server_sk)?;
    let sig = sk.sign(new_envelope.as_bytes());

    // Aggregate signature
    /*
    let agg_sig = aggregate_sigs_multi(&[
        (&client_sig, &client_pk),
        (&sig, &sk.public_key()),
    ]);
    */

    // Send our signature to the client for aggregation. Client needs to re-sign
    // with the new envelope that includes metadata.
    Ok(Json(SignResponse {
        new_envelope: serde_json::to_vec(&envelope)?,
        server_sig: sig.as_bytes(),
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
