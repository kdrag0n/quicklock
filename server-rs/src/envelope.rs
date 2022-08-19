use crate::error::AppResult;
use crate::serialize::base64 as serde_b64;

use anyhow::anyhow;
use chacha20poly1305::{aead::{Aead, Nonce}, XChaCha20Poly1305, KeyInit};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditStamp {
    #[serde(with = "serde_b64")]
    pub envelope_hash: Vec<u8>,
    pub client_ip: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestEnvelope {
    // XChaCha20-Poly1305
    #[serde(rename = "p", with = "serde_b64")]
    pub enc_payload: Vec<u8>,
    #[serde(rename = "n", with = "serde_b64")]
    pub enc_nonce: Vec<u8>, // 192 bits
}

impl RequestEnvelope {
    pub fn seal_raw(payload: &[u8], enc_key: &[u8]) -> AppResult<RequestEnvelope> {
        let cipher = XChaCha20Poly1305::new_from_slice(enc_key)
            .map_err(|_| anyhow!("Bad key"))?;
        let nonce_bytes: [u8; 24] = rand::random();
        let nonce = <Nonce<XChaCha20Poly1305>>::from_slice(&nonce_bytes);
        let enc_payload = cipher.encrypt(nonce, payload)
            .map_err(|_| anyhow!("Failed to encrypt"))?;

        Ok(RequestEnvelope {
            enc_payload,
            enc_nonce: nonce_bytes.into(),
        })
    }

    pub fn seal<T>(request: &T, enc_key: &[u8]) -> AppResult<RequestEnvelope>
    where
        T: Serialize
    {
        let payload = serde_json::to_string(request)?;
        Self::seal_raw(payload.as_bytes(), enc_key)
    }

    pub fn open_raw(&self, enc_key: &[u8]) -> AppResult<Vec<u8>> {
        // Decrypt request contents
        let cipher = XChaCha20Poly1305::new_from_slice(enc_key)
            .map_err(|_| anyhow!("Bad key"))?;
        let nonce = <Nonce<XChaCha20Poly1305>>::from_slice(&self.enc_nonce);
        let payload = cipher.decrypt(nonce, &*self.enc_payload)
            .map_err(|_| anyhow!("Failed to decrypt"))?;

        Ok(payload)
    }

    pub fn open<T>(&self, enc_key: &[u8]) -> AppResult<T>
    where
        T: DeserializeOwned
    {
        let payload = self.open_raw(enc_key)?;

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
pub struct SignedRequestEnvelope {
    pub device_id: String,
    pub envelope: RequestEnvelope,
    #[serde(with = "serde_b64")]
    pub client_signature: Vec<u8>,
    pub audit_stamp: AuditStamp,
    #[serde(with = "serde_b64")]
    pub audit_signature: Vec<u8>,
}
