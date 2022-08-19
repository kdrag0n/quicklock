use lazy_static::lazy_static;
use tracing::log::debug;
use crate::envelope::SignedRequestEnvelope;
use crate::error::AppResult;
use crate::lock::model::{UnlockChallenge, UnlockStartRequest};
use std::net::SocketAddr;
use std::time::Duration;
use anyhow::anyhow;
use axum::{Extension, Json, Router};
use axum::extract::{Path, ConnectInfo};
use axum::response::IntoResponse;
use axum::routing::post;
use dashmap::DashMap;
use tracing::{error, info};
use tokio::time::sleep;
use crate::checks::require;
use super::config::CONFIG;
use super::homeassistant;
use crate::time::now;
use super::request::EnvelopeOpen;
use super::crypto::generate_secret;
use super::store::STORE;

lazy_static! {
    static ref UNLOCK_CHALLENGES: DashMap<String, UnlockChallenge> = DashMap::new();
}

pub fn start_unlock(req: UnlockStartRequest) -> AppResult<UnlockChallenge> {
    debug!("Start unlock: {:?}", req);

    CONFIG.entities.get(&req.entity_id)
        .ok_or_else(|| anyhow!("Entity not found"))?;

    let challenge = UnlockChallenge {
        id: generate_secret(),
        timestamp: now(),
        entity_id: req.entity_id.clone(),
    };
    UNLOCK_CHALLENGES.insert(challenge.id.clone(), challenge.clone());

    Ok(challenge)
}

async fn start_unlock_route(Json(req): Json<UnlockStartRequest>) -> AppResult<impl IntoResponse> {
    Ok(Json(start_unlock(req)?))
}

pub fn finish_unlock(
    envelope: SignedRequestEnvelope,
    id: String,
    addr: SocketAddr,
) -> AppResult<UnlockChallenge> {
    // Envelope only contains challenge nonce to minimize wire size, not entire UnlockChallenge
    let req_challenge = envelope.open_raw(&addr)?;
    debug!("Finish unlock: {:?}", req_challenge);

    let (_, challenge) = UNLOCK_CHALLENGES.remove(&id)
        .ok_or_else(|| anyhow!("Challenge not found"))?;
    require(req_challenge == base64::decode(&challenge.id)?)?;

    STORE.get_device_for_entity(&envelope.device_id, &challenge.entity_id)
        .ok_or_else(|| anyhow!("Entity not allowed"))?;

    // Verify timestamp
    require((now() - challenge.timestamp) <= CONFIG.time_grace_period)?;

    Ok(challenge)
}

pub async fn finish_unlock_route(
    Json(envelope): Json<SignedRequestEnvelope>,
    Path(id): Path<String>,
    client: Extension<reqwest::Client>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> AppResult<impl IntoResponse> {
    let challenge = finish_unlock(envelope, id, addr)?;

    // Unlock
    info!("Posting HA unlock");
    // homeassistant::post_lock(&client, true, &challenge.entity_id).await?;

    // Re-lock after delay
    tokio::spawn(async move {
        sleep(Duration::from_millis(CONFIG.relock_delay)).await;
        info!("Posting HA lock");
        if let Err(e) = homeassistant::post_lock(&client, false, &challenge.entity_id).await {
            error!("Failed to re-lock: {}", e);
        }
    });

    Ok(Json(()))
}

pub fn service() -> Router {
    let client = reqwest::Client::new();

    Router::new()
        .route("/start", post(start_unlock_route))
        .route("/:challenge_id/finish", post(finish_unlock_route))
        .layer(Extension(client))
}
