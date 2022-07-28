use actix_web::{web, Scope, Responder, post};
use actix_web::web::{Data, Json, Path};
use lazy_static::lazy_static;
use qlock::error::AppResult;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Duration;
use anyhow::anyhow;
use futures_timer::Delay;
use log::{error, info};
use qlock::checks::require;
use qlock::time::now;
use crate::{CONFIG, homeassistant};
use crate::crypto::generate_secret;
use crate::store::DataStore;

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
    static ref UNLOCK_CHALLENGES: Mutex<HashMap<String, UnlockChallenge>> = Mutex::new(HashMap::new());
}

#[post("start")]
async fn start_unlock(req: Json<UnlockStartRequest>) -> AppResult<impl Responder> {
    println!("Start unlock: {:?}", req);

    CONFIG.entities.get(&req.entity_id)
        .ok_or(anyhow!("Entity not found"))?;

    let challenge = UnlockChallenge {
        id: generate_secret(),
        timestamp: now(),
        entity_id: req.entity_id.clone(),
    };
    UNLOCK_CHALLENGES.lock().unwrap().insert(challenge.id.clone(), challenge.clone());

    Ok(Json(challenge))
}

#[post("{challenge_id}/finish")]
async fn finish_unlock(
    req: Json<UnlockFinishRequest>,
    path: Path<(String,)>,
    client: Data<awc::Client>,
) -> AppResult<impl Responder> {
    println!("Finish unlock: {:?}", req);

    let (id,) = path.into_inner();
    let challenge = UNLOCK_CHALLENGES.lock().unwrap().remove(&id)
        .ok_or(anyhow!("Challenge not found"))?;

    DataStore::get().get_device_for_entity(&req.public_key, &challenge.entity_id)
        .ok_or(anyhow!("Device not found or not allowed"))?;

    // Verify timestamp
    require((now() - challenge.timestamp) <= CONFIG.time_grace_period)?;

    // Unlock
    info!("Posting HA unlock");
    homeassistant::post_lock(&client, true, &challenge.entity_id).await?;

    // Re-lock after delay
    actix_rt::spawn(async move {
        Delay::new(Duration::from_millis(CONFIG.relock_delay)).await;
        info!("Posting HA lock");
        if let Err(e) = homeassistant::post_lock(&client, false, &challenge.entity_id).await {
            error!("Failed to re-lock: {}", e);
        }
    });

    Ok(Json(()))
}

pub fn service() -> Scope {
    let client = awc::ClientBuilder::new()
        .add_default_header(("Content-Type", "application/json"))
        .add_default_header(("Authorization", format!("Bearer {}", CONFIG.ha_api_key)))
        .finish();

    web::scope("unlock")
        .app_data(Data::new(client))
        .service(start_unlock)
        .service(finish_unlock)
}
