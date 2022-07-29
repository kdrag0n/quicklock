use anyhow::anyhow;
use qlock::error::AppResult;
use crate::CONFIG;
use std::str;
use reqwest::Client;

pub async fn post_lock(client: &Client, unlocked: bool, entity_id: &str) -> AppResult<()> {
    let service = if unlocked { "unlock" } else { "lock" };
    let entity = CONFIG.entities.get(entity_id)
        .ok_or(anyhow!("Entity not found"))?;

    let req = serde_json::json!({
        "entity_id": entity.ha_entity,
    });

    let resp = client.post(format!("http://192.168.20.137:8123/api/services/lock/{}", service))
        .header("Authorization", format!("Bearer {}", CONFIG.ha_api_key))
        .json(&req)
        .send()
        .await
        .map_err(|_| anyhow!("Failed to call lock service"))?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Lock service returned {}", resp.status()).into())
    }
}
