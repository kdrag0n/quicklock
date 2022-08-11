use std::net::SocketAddr;

use anyhow::anyhow;
use crate::{error::AppResult, envelope::SignedRequestEnvelope, checks::{require, require_eq}, time::now};
use serde::de::DeserializeOwned;
use crate::lock::config::CONFIG;
use crate::lock::crypto::{verify_bls_signature_str, verify_ec_signature_str};
use super::store::STORE;

pub trait EnvelopeOpen {
    fn open<T: DeserializeOwned>(&self, addr: &SocketAddr) -> AppResult<T>;
    fn open_for_delegation<T: DeserializeOwned>(&self, addr: &SocketAddr) -> AppResult<T>;
}

fn _open<T>(env: &SignedRequestEnvelope, is_delegation: bool, addr: &SocketAddr) -> AppResult<T>
where
    T: DeserializeOwned,
{
    let device = STORE.get_device(&env.device_id)
        .ok_or_else(|| anyhow!("Device not found"))?;

    // Verify signature of envelope JSON string
    let envelope_str = serde_json::to_string(&env.envelope)?;
    let ec_pk = if is_delegation { &device.delegation_key } else { &device.public_key };
    verify_ec_signature_str(&envelope_str, ec_pk, &env.ec_signature)?;

    if let Some(bls_pk) = device.bls_public_key {
        verify_bls_signature_str(&envelope_str, &bls_pk, &env.bls_signature)?;
    }

    // Verify public request metadata
    let meta = env.envelope.public_metadata.as_ref()
        .ok_or_else(|| anyhow!("Missing public metadata"))?;
    require_eq(&meta.client_ip, &addr.ip().to_string())?;
    require(now().abs_diff(meta.timestamp) <= CONFIG.time_grace_period)?;

    env.envelope.open(&device.enc_key)
}

impl EnvelopeOpen for SignedRequestEnvelope {
    fn open<T: DeserializeOwned>(&self, addr: &SocketAddr) -> AppResult<T> {
        _open(self, false, addr)
    }

    fn open_for_delegation<T: DeserializeOwned>(&self, addr: &SocketAddr) -> AppResult<T> {
        _open(self, true, addr)
    }
}
