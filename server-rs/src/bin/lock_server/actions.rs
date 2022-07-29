use lazy_static::lazy_static;
use qlock::error::AppResult;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use anyhow::anyhow;
use axum::{Extension, Json, Router};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::post;
use dashmap::DashMap;
use log::{error, info};
use tokio::time::sleep;
use qlock::checks::require;
use qlock::time::now;
use crate::{CONFIG, homeassistant};
use crate::crypto::generate_secret;
use crate::store::{DataStore, STORE};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnlockChallenge {
    id: String,
    timestamp: u64,
    entity_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnlockStartRequest {
    entity_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnlockFinishRequest {
    // Challenge ID is in URL
    public_key: String,
    bls_signature: String,
    ec_signature: String,
}

lazy_static! {
    static ref UNLOCK_CHALLENGES: DashMap<String, UnlockChallenge> = DashMap::new();
}

async fn start_unlock(req: Json<UnlockStartRequest>) -> AppResult<impl IntoResponse> {
    println!("Start unlock: {:?}", req);

    CONFIG.entities.get(&req.entity_id)
        .ok_or(anyhow!("Entity not found"))?;

    let challenge = UnlockChallenge {
        id: generate_secret(),
        timestamp: now(),
        entity_id: req.entity_id.clone(),
    };
    UNLOCK_CHALLENGES.insert(challenge.id.clone(), challenge.clone());

    Ok(Json(challenge))
}

async fn finish_unlock(
    req: Json<UnlockFinishRequest>,
    Path(id): Path<String>,
    client: Extension<reqwest::Client>,
) -> AppResult<impl IntoResponse> {
    println!("Finish unlock: {:?}", req);

    let (_, challenge) = UNLOCK_CHALLENGES.remove(&id)
        .ok_or(anyhow!("Challenge not found"))?;

    STORE.get_device_for_entity(&req.public_key, &challenge.entity_id)
        .ok_or(anyhow!("Device not found or not allowed"))?;

    // Verify timestamp
    require((now() - challenge.timestamp) <= CONFIG.time_grace_period)?;

    // Unlock
    info!("Posting HA unlock");
    homeassistant::post_lock(&client, true, &challenge.entity_id).await?;

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
        .route("/start", post(start_unlock))
        .route("/:challenge_id/finish", post(finish_unlock))
        .layer(Extension(client))
}
