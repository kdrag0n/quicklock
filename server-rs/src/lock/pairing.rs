use std::net::SocketAddr;

use anyhow::anyhow;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::extract::{Path, ConnectInfo};
use axum::routing::{get, post};
use dashmap::DashMap;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use tracing::log::{debug, info};
use crate::crypto::hash;
use crate::envelope::SignedRequestEnvelope;
use crate::lock::model::{PairFinishPayload, PairingChallenge, InitialPairQr, InitialPairFinishRequest, Delegation};
use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;
use ring::hmac;
use crate::checks::require;
use super::request::EnvelopeOpen;
use super::store::{PairedDevice, STORE};
use crate::error::{AppResult, Error, HttpError};
use crate::time::now;
use super::attestation::verify_chain;
use super::CONFIG;
use super::crypto::generate_secret;

lazy_static! {
    static ref PAIRING_CHALLENGES: DashMap<String, PairingChallenge> = DashMap::new();
    static ref FINISH_PAYLOADS: DashMap<String, String> = DashMap::new();
    static ref INITIAL_PAIRING_SECRET: Mutex<Option<String>> = Mutex::new(None);
}

trait Validate {
    fn validate(&self) -> AppResult<()>;
}
impl Validate for PairingChallenge {
    fn validate(&self) -> AppResult<()> {
        require(now() - self.timestamp <= CONFIG.time_grace_period)?;
        Ok(())
    }
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

    // Max expiry = delegator's expiry
    let expires_at = if let Some(ref delegator) = delegated_by {
        let delegator = STORE.get_device(delegator)
            .ok_or_else(|| anyhow!("Missing device"))?;
        expires_at.min(delegator.expires_at)
    } else {
        expires_at
    };

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
                .and_then(|d| d.allowed_entities)),
        None => allowed_entities,
    };

    // Enroll
    let pk_bytes = base64::decode(&req.public_key)?;
    STORE.add_device(PairedDevice {
        id: base64::encode(&hash(&pk_bytes)),
        public_key: req.public_key.clone(),
        delegation_key: req.delegation_key.clone(),
        enc_key: req.enc_key.clone(),
        audit_public_key: req.audit_public_key.clone(),
        // Params
        expires_at,
        delegated_by,
        allowed_entities,
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
    debug!("secret = {}", secret);
    INITIAL_PAIRING_SECRET.lock().replace(secret.clone());

    // Print QR code
    let qr_data = serde_json::to_string(&InitialPairQr {
        secret,
    })?;
    let qr = QrCode::new(qr_data)?;
    let image = qr.render::<Dense1x2>().build();
    info!("{}", image);

    Ok(Json(()))
}

async fn finish_initial(req: Json<InitialPairFinishRequest>) -> AppResult<impl IntoResponse> {
    debug!("Finish initial pair: {:?}", req);

    // Verify HMAC
    let secret = INITIAL_PAIRING_SECRET.lock().take().unwrap();
    let key_bytes = base64::decode(&secret)?;
    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);
    hmac::verify(&key, req.finish_payload.as_bytes(), &base64::decode(&req.mac)?)?;

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
    debug!("Delegated pair finish payload: {:?}", payload);
    FINISH_PAYLOADS.insert(id, payload);
    Ok(Json(()))
}

async fn finish_delegated(
    Path(id): Path<String>,
    envelope: Json<SignedRequestEnvelope>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> AppResult<impl IntoResponse> {
    let req: Delegation = envelope.open_for_delegation(&addr)?;
    debug!("Finish delegated pair: {:?}", req);

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
