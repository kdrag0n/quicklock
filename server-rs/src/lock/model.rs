use serde::{Deserialize, Serialize};

use crate::serialize::base64 as serde_b64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UnlockChallenge {
    pub id: String,
    pub timestamp: u64,
    pub entity_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockStartRequest {
    #[serde(rename = "e")]
    pub entity_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub ha_entity: String,
}

/*
 * Pairing
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingChallenge {
    pub id: String,
    pub timestamp: u64,
    pub is_initial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialPairQr {
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairFinishPayload {
    pub challenge_id: String,
    pub public_key: String,
    pub delegation_key: String,
    #[serde(with = "serde_b64")]
    pub enc_key: Vec<u8>,
    #[serde(with = "serde_b64")]
    pub audit_public_key: Vec<u8>,
    pub main_attestation_chain: Vec<String>,
    pub delegation_attestation_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitialPairFinishRequest {
    pub finish_payload: String,
    pub mac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Delegation {
    pub finish_payload: String,
    pub expires_at: u64,
    pub allowed_entities: Option<Vec<String>>,
}
