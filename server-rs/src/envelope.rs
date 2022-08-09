use crate::error::AppResult;
use crate::serialize::base64 as serde_b64;

use anyhow::anyhow;
use chacha20poly1305::{aead::{Aead, Nonce}, XChaCha20Poly1305, KeyInit};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::str;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPublicMetadata {
    pub client_ip: String,
    pub timestamp: u64,
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
pub struct SignedRequestEnvelope {
    pub device_id: String,
    pub envelope: RequestEnvelope,
    pub bls_signature: String,
    pub ec_signature: String,
}
