use std::collections::HashMap;
use std::process::Command;
use actix_web::{Scope, web, get, post, Responder, HttpRequest};
use actix_web::web::{Json, Path, Query};
use anyhow::anyhow;
use dashmap::DashMap;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use qrcode::QrCode;
use qrcode::render::unicode::{Canvas1x2, Dense1x2};
use ring::hmac;
use qlock::checks::require;
use crate::store::{DataStore, PairedDevice, STORE};
use qlock::error::{AppResult, Error, HttpError};
use serde::{Serialize, Deserialize};
use qlock::time::now;
use crate::attestation::verify_chain;
use crate::CONFIG;
use crate::crypto::{generate_secret, verify_signature_str};

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
    pub bls_public_keys: Option<Vec<String>>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignedDelegation {
    pub device: String,
    pub delegation: String,
    pub bls_signature: String,
    pub ec_signature: String,
}

fn finish_pair(
    req: &PairFinishPayload,
    delegated_by: Option<String>,
    expires_at: u64,
    allowed_entities: Option<Vec<String>>,
) -> Result<(), Error> {
    let challenge = PAIRING_CHALLENGES.remove(&req.challenge_id)
        .map(|c| c.1.clone())
        .ok_or(HttpError::NotFound)?;
    // Drop challenge data
    FINISH_PAYLOADS.remove(&challenge.id);

    require(challenge.is_initial == delegated_by.is_none())?;

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
                .map(|d| d.allowed_entities)
                .flatten())
        ,
        None => allowed_entities,
    };

    STORE.add_device(PairedDevice {
        public_key: req.public_key.clone(),
        delegation_key: req.delegation_key.clone(),
        bls_public_keys: req.bls_public_keys.clone(),
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
#[post("initial/start")]
async fn start_initial() -> AppResult<impl Responder> {
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

#[post("initial/finish")]
async fn finish_initial(req: Json<InitialPairFinishRequest>) -> AppResult<impl Responder> {
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
#[get("delegated/{challenge_id}/finish_payload")]
async fn get_finish_payload(path: Path<(String,)>) -> AppResult<impl Responder> {
    let (challenge_id,) = path.into_inner();
    let payload = FINISH_PAYLOADS.get(&challenge_id)
        .map(|p| p.clone())
        .ok_or(HttpError::NotFound)?;
    Ok(payload)
}

#[post("delegated/{challenge_id}/finish_payload")]
async fn post_finish_payload(
    path: Path<(String,)>,
    payload: String,
) -> AppResult<impl Responder> {
    let (id,) = path.into_inner();
    require(PAIRING_CHALLENGES.contains_key(&id))?;
    require(!FINISH_PAYLOADS.contains_key(&id))?;

    // Raw string for flexibility
    println!("Delegated pair finish payload: {:?}", payload);
    FINISH_PAYLOADS.insert(id, payload);
    Ok(Json(()))
}

#[post("delegated/{challenge_id}/finish")]
async fn finish_delegated(
    path: Path<(String,)>,
    req: Json<SignedDelegation>,
) -> AppResult<impl Responder> {
    println!("Finish delegated pair: {:?}", req);
    let device = STORE.get_device(&req.device)
        .ok_or(anyhow!("Device not found"))?;
    verify_signature_str(&req.delegation, &device.delegation_key, &req.ec_signature)?;

    let del: Delegation = serde_json::from_str(&req.delegation)?;

    // Prevent delegator from changing request
    let (id,) = path.into_inner();
    {
        let orig_payload = FINISH_PAYLOADS.get(&id)
            .ok_or(HttpError::NotFound)?;
        require(*orig_payload == del.finish_payload)?;
    }

    let payload: PairFinishPayload = serde_json::from_str(&del.finish_payload)?;
    finish_pair(
        &payload,
        Some(req.device.clone()),
        del.expires_at,
        del.allowed_entities,
    )?;

    Ok(Json(()))
}

/*
 * Common
 */
#[post("get_challenge")]
async fn get_challenge() -> AppResult<impl Responder> {
    let challenge = PairingChallenge {
        id: generate_secret(),
        timestamp: now(),
        is_initial: !STORE.has_paired_devices(),
    };

    PAIRING_CHALLENGES.insert(challenge.id.clone(), challenge.clone());
    println!("challenge = {:?}", challenge);
    Ok(Json(challenge))
}

pub fn service() -> Scope {
    web::scope("/pair")
        .service(start_initial)
        .service(finish_initial)
        .service(get_finish_payload)
        .service(post_finish_payload)
        .service(finish_delegated)
        .service(get_challenge)
}
