use std::net::SocketAddr;

use qlock::{audit::RequestEnvelope, error::AppResult, checks::require, time::now};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use anyhow::anyhow;

use crate::{store::STORE, crypto::{verify_ec_signature_str, verify_bls_signature_str}, config::CONFIG};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedRequestEnvelope {
    pub device_id: String,
    pub envelope: RequestEnvelope,
    pub bls_signature: String,
    pub ec_signature: String,
}

impl SignedRequestEnvelope {
    fn _open<T>(&self, is_delegation: bool, addr: &SocketAddr) -> AppResult<T>
    where
        T: DeserializeOwned,
    {
        let device = STORE.get_device(&self.device_id)
            .ok_or_else(|| anyhow!("Device not found"))?;

        // Verify signature of envelope JSON string
        let envelope_str = serde_json::to_string(&self.envelope)?;
        let ec_pk = if is_delegation { &device.delegation_key } else { &device.public_key };
        verify_ec_signature_str(&envelope_str, ec_pk, &self.ec_signature)?;
    
        if let Some(bls_pk) = device.bls_public_key {
            verify_bls_signature_str(&envelope_str, &bls_pk, &self.bls_signature)?;
        }

        // Verify public request metadata
        let meta = self.envelope.public_metadata.as_ref()
            .ok_or_else(|| anyhow!("Missing public metadata"))?;
        require(meta.client_ip == addr.to_string())?;
        require(now().abs_diff(meta.timestamp) <= CONFIG.time_grace_period)?;

        self.envelope.open(&device.enc_key)
    }

    pub fn open<T: DeserializeOwned>(&self, addr: &SocketAddr) -> AppResult<T> {
        self._open(false, addr)
    }

    pub fn open_for_delegation<T: DeserializeOwned>(&self, addr: &SocketAddr) -> AppResult<T> {
        self._open(true, addr)
    }
}
