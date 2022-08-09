use std::fmt::Debug;
use std::net::SocketAddr;

use anyhow::anyhow;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::extract::{Path, ConnectInfo};
use axum::routing::{get, post};
use dashmap::DashMap;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;
use ring::hmac;
use qlock::checks::require;
use crate::request::SignedRequestEnvelope;
use crate::store::{PairedDevice, STORE};
use qlock::error::{AppResult, Error, HttpError};
use serde::{Serialize, Deserialize};
use qlock::time::now;
use crate::attestation::verify_chain;
use crate::CONFIG;
use crate::crypto::{generate_secret};
use qlock::serialize::base64 as serde_b64;

lazy_static! {
    static ref PAIRING_CHALLENGES: DashMap<String, PairingChallenge> = DashMap::new();
    static ref FINISH_PAYLOADS: DashMap<String, String> = DashMap::new();
    static ref INITIAL_PAIRING_SECRET: Mutex<Option<String>> = Mutex::new(None);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairingChallenge {
    id: String,
    timestamp: u64,
    is_initial: bool,
}
impl PairingChallenge {
    fn validate(&self) -> Result<(), Error> {
        require(now() - self.timestamp <= CONFIG.time_grace_period)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitialPairQr {
    secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairFinishPayload {
    pub challenge_id: String,
    pub public_key: String,
    pub delegation_key: String,
    #[serde(with = "serde_b64")]
    pub enc_key: Vec<u8>,
    pub bls_public_key: Option<String>,
    pub main_attestation_chain: Vec<String>,
    pub delegation_attestation_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitialPairFinishRequest {
    pub finish_payload: String,
    pub mac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Delegation {
    pub finish_payload: String,
    pub expires_at: u64,
    pub allowed_entities: Option<Vec<String>>,
}

fn finish_pair(
    req: &PairFinishPayload,
    delegated_by: Option<String>,
    expires_at: u64,
    allowed_entities: Option<Vec<String>>,
) -> Result<(), Error> {
    let challenge = PAIRING_CHALLENGES.remove(&req.challenge_id)
        .map(|(_, challenge)| challenge)
        .ok_or(HttpError::NotFound)?;
    // Drop challenge data
    FINISH_PAYLOADS.remove(&challenge.id);

    require(challenge.is_initial == delegated_by.is_none())?;
    require(req.enc_key.len() == 32)?;

    // Verify timestamp
    challenge.validate()?;

    // Verify main attestation and certificate chain
    verify_chain(&req.main_attestation_chain, &challenge.id, false)?;

    // Verify delegation attestation and certificate chain
    verify_chain(&req.delegation_attestation_chain, &challenge.id, true)?;

    // Only allow entities that delegator has access to
    let allowed_entities = match delegated_by {
        Some(ref delegator) => allowed_entities
            // Filter allowed list if given
            .map(|entities| {
                entities.into_iter()
                    .filter(|e| STORE.get_device_for_entity(delegator, e).is_some())
                    .collect()
            })
            // Otherwise limit to delegator's allowed list
            .or_else(|| STORE.get_device(delegator)
                .and_then(|d| d.allowed_entities))
        ,
        None => allowed_entities,
    };

    STORE.add_device(PairedDevice {
        public_key: req.public_key.clone(),
        delegation_key: req.delegation_key.clone(),
        enc_key: req.enc_key.clone(),
        bls_public_key: req.bls_public_key.clone(),
        // Params
        expires_at,
        delegated_by,
        allowed_entities,
        serialized_authenticator: None
    });
    Ok(())
}

/*
 * Initial
 */
async fn start_initial() -> AppResult<impl IntoResponse> {
    // Only for initial setup
    require(!STORE.has_paired_devices())?;
    require(INITIAL_PAIRING_SECRET.lock().is_none())?;

    // Generate secret
    let secret = generate_secret();
    println!("secret = {}", secret);
    INITIAL_PAIRING_SECRET.lock().replace(secret.clone());

    // Print QR code
    let qr_data = serde_json::to_string(&InitialPairQr {
        secret,
    })?;
    let qr = QrCode::new(qr_data)?;
    let image = qr.render::<Dense1x2>().build();
    println!("{}", image);

    Ok(Json(()))
}

async fn finish_initial(req: Json<InitialPairFinishRequest>) -> AppResult<impl IntoResponse> {
    println!("Finish initial pair: {:?}", req);

    // Verify HMAC
    let secret = INITIAL_PAIRING_SECRET.lock().take().unwrap();
    let key_bytes = base64::decode(&secret)?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);
    hmac::verify(&key, req.finish_payload.as_bytes(), &base64::decode(&req.mac)?)
        .map_err(|_| anyhow!("Invalid MAC"))?;

    let payload: PairFinishPayload = serde_json::from_str(&req.finish_payload)?;
    finish_pair(&payload, None, u64::MAX, None)?;

    Ok(Json(()))
}

/*
 * Delegation
 */
async fn get_finish_payload(Path(challenge_id): Path<String>) -> AppResult<impl IntoResponse> {
    let payload = FINISH_PAYLOADS.get(&challenge_id)
        .map(|p| p.clone())
        .ok_or(HttpError::NotFound)?;
    Ok(payload)
}

async fn post_finish_payload(
    Path(id): Path<String>,
    payload: String,
) -> AppResult<impl IntoResponse> {
    require(PAIRING_CHALLENGES.contains_key(&id))?;
    require(!FINISH_PAYLOADS.contains_key(&id))?;

    // Raw string for flexibility
    println!("Delegated pair finish payload: {:?}", payload);
    FINISH_PAYLOADS.insert(id, payload);
    Ok(Json(()))
}

async fn finish_delegated(
    Path(id): Path<String>,
    envelope: Json<SignedRequestEnvelope>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> AppResult<impl IntoResponse> {
    let req: Delegation = envelope.open_for_delegation(&addr)?;
    println!("Finish delegated pair: {:?}", req);

    // Prevent delegator from changing request
    {
        let orig_payload = FINISH_PAYLOADS.get(&id)
            .ok_or(HttpError::NotFound)?;
        require(*orig_payload == req.finish_payload)?;
    }

    let payload: PairFinishPayload = serde_json::from_str(&req.finish_payload)?;
    finish_pair(
        &payload,
        Some(envelope.device_id.clone()),
        req.expires_at,
        req.allowed_entities,
    )?;

    Ok(Json(()))
}

/*
 * Common
 */
async fn get_challenge() -> impl IntoResponse {
    let challenge = PairingChallenge {
        id: generate_secret(),
        timestamp: now(),
        is_initial: !STORE.has_paired_devices(),
    };

    PAIRING_CHALLENGES.insert(challenge.id.clone(), challenge.clone());
    Json(challenge)
}

pub fn service() -> Router {
    Router::new()
        .route("/initial/start", post(start_initial))
        .route("/initial/finish", post(finish_initial))
        .route("/delegated/:challenge_id/finish_payload", get(get_finish_payload))
        .route("/delegated/:challenge_id/finish_payload", post(post_finish_payload))
        .route("/delegated/:challenge_id/finish", post(finish_delegated))
        .route("/get_challenge", post(get_challenge))
}
