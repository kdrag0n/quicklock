use std::fs::File;
use std::io;
use std::io::BufReader;
use std::time::SystemTime;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tracing::{debug};
use crate::serialize::{base64 as serde_b64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    pub client_pk: String,
    #[serde(with="serde_b64")]
    pub server_sk: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub id: String,
    pub timestamp: SystemTime,
    #[serde(with="serde_b64")]
    pub enc_message: Vec<u8>,
    #[serde(with="serde_b64")]
    pub enc_nonce: Vec<u8>,
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
        self.devices.insert(device.client_pk.clone(), device);
        self.persist();
    }

    pub fn get_device(&self, public_key: &String) -> Option<PairedDevice> {
        self.devices.get(public_key).map(|d| d.clone())
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
        self.logs.get(device_id)
            .map(|v| v.clone())
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
