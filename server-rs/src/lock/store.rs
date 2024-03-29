use crate::serialize::base64 as serde_b64;

use std::fs::File;
use std::io;
use std::io::BufReader;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use crate::time::now;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    // Hash of PK (fingerprint)
    pub id: String,

    // For normal actions (unlock)
    // Custom protocol w/ auditing: this is the ECDSA key
    pub public_key: String,
    // For adding a new device. This one requires protected confirmation, verified by attestation
    pub delegation_key: String,
    // For decrypting request envelopes
    #[serde(with = "serde_b64")]
    pub enc_key: Vec<u8>,
    // When this device's access expires (for temporary access)
    pub expires_at: u64,
    // Device that authorized this. Null for initial setup
    pub delegated_by: Option<String>,
    // Null for all
    pub allowed_entities: Option<Vec<String>>,

    // Custom protocol
    // Audit server's Ed25519 key
    #[serde(with = "serde_b64")]
    pub audit_public_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStore {
    devices: DashMap<String, PairedDevice>,
}

pub static STORE: Lazy<DataStore> = Lazy::new(DataStore::new);

impl DataStore {
    fn new() -> Self {
        load_data().unwrap_or(Self {
            devices: DashMap::new(),
        })
    }

    pub fn add_device(&self, device: PairedDevice) {
        self.devices.insert(device.id.clone(), device);
        self.persist();
    }

    pub fn get_device(&self, public_key: &String) -> Option<PairedDevice> {
        let device = self.devices.get(public_key)?.clone();
        if device.expires_at > now() {
            Some(device)
        } else {
            None
        }
    }

    pub fn get_device_for_entity(&self, public_key: &String, entity_id: &String) -> Option<PairedDevice> {
        self.get_device(public_key)
            .filter(|d| {
                match d.allowed_entities {
                    Some(ref entities) => entities.contains(entity_id),
                    None => true,
                }
            })
    }

    pub fn has_paired_devices(&self) -> bool {
        !self.devices.is_empty()
    }

    fn persist(&self) {
        let file = File::create("state_lock.json").unwrap();
        let mut writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self).unwrap();
    }
}

fn load_data() -> Result<DataStore, io::Error> {
    let f = File::open("state_lock.json")?;
    let reader = BufReader::new(f);
    Ok(serde_json::from_reader(reader)?)
}
