use std::net::SocketAddr;

use anyhow::anyhow;
use ring::constant_time::verify_slices_are_equal;
use crate::{error::AppResult, envelope::SignedRequestEnvelope, checks::{require, require_eq}, time::now};
use serde::de::DeserializeOwned;
use crate::crypto::hash;
use crate::lock::config::CONFIG;
use crate::lock::crypto::{verify_ec_signature_str, verify_ed25519_signature};
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

    // Verify client's ECDSA signature for envelope
    let envelope_str = serde_json::to_string(&env.envelope)?;
    let ec_pk = if is_delegation { &device.delegation_key } else { &device.public_key };
    verify_ec_signature_str(&envelope_str, ec_pk, &env.client_signature)?;

    // Verify audit server's Ed25519 signature for stamp
    let stamp = &env.audit_stamp;
    let stamp_data = serde_json::to_vec(stamp)?;
    verify_ed25519_signature(&stamp_data, &device.audit_public_key, &env.audit_signature)?;

    // Verify stamp
    verify_slices_are_equal(&stamp.envelope_hash, &hash(envelope_str.as_bytes()))?;

    // Verify public request metadata
    require_eq(&stamp.client_ip, &addr.ip().to_string())?;
    require(now().abs_diff(stamp.timestamp) <= CONFIG.time_grace_period)?;

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
