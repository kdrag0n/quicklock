use std::fs::File;
use std::io;
use std::io::BufReader;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tracing::{debug};
use crate::{serialize::{base64 as serde_b64}, envelope::AuditStamp};

use super::RequestEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    pub id: String,
    // For auth
    #[serde(with = "serde_b64")]
    pub client_mac_key: Vec<u8>,
    // For signing request
    #[serde(with = "serde_b64")]
    pub server_keypair: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub id: String,
    pub envelope: RequestEnvelope,
    pub stamp: AuditStamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStore {
    devices: DashMap<String, PairedDevice>,
    logs: DashMap<String, Vec<LogEvent>>,
}

pub static STORE: Lazy<DataStore> = Lazy::new(DataStore::create);

impl DataStore {
    fn create() -> Self {
        load_data().unwrap_or(Self {
            devices: DashMap::new(),
            logs: DashMap::new(),
        })
    }

    pub fn add_device(&self, device: PairedDevice) {
        self.devices.insert(device.id.clone(), device);
        self.persist();
    }

    pub fn get_device(&self, id: &String) -> Option<PairedDevice> {
        Some(self.devices.get(id)?.clone())
    }

    pub fn log_event(&self, device_id: &str, event: LogEvent) {
        debug!("Log event: {:?}", event);
        // Persist needs the lock
        {
            let mut entries = self.logs.entry(device_id.into()).or_default();
            entries.push(event);
        }
        self.persist();
    }

    pub fn get_logs(&self, device_id: &String) -> Option<Vec<LogEvent>> {
        Some(self.logs.get(device_id)?.clone())
    }

    fn persist(&self) {
        let file = File::create("state_audit.json").unwrap();
        let mut writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self).unwrap();
    }
}

fn load_data() -> Result<DataStore, io::Error> {
    let f = File::open("state_audit.json")?;
    let reader = BufReader::new(f);
    Ok(serde_json::from_reader(reader)?)
}
