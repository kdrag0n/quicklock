use crate::serialize::base64 as serde_b64;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditClientState {
    #[serde(with = "serde_b64")]
    pub bls_sk: Vec<u8>,
    #[serde(with = "serde_b64")]
    pub enc_key: Vec<u8>,
}
