use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::sync::{Mutex, MutexGuard};
use std::time::SystemTime;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use qlock::serialize::{base64 as serde_b64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    pub client_pk: String,
    #[serde(with="serde_b64")]
    pub server_sk: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthEvent {
    pub id: String,
    pub timestamp: SystemTime,
    #[serde(with="serde_b64")]
    pub payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataStore {
    devices: HashMap<String, PairedDevice>,
    logs: HashMap<String, Vec<AuthEvent>>,
}

static STORE: Lazy<Mutex<DataStore>> = Lazy::new(|| Mutex::new(DataStore::create()));

impl DataStore {
    pub fn get() -> MutexGuard<'static, DataStore> {
        STORE.lock().unwrap()
    }

    fn create() -> DataStore {
        load_data().unwrap_or(DataStore {
            devices: HashMap::new(),
            logs: HashMap::new(),
        })
    }

    pub fn add_device(&mut self, device: PairedDevice) {
        self.devices.insert(device.client_pk.clone(), device);
        self.persist();
    }

    pub fn get_device(&self, public_key: &String) -> Option<PairedDevice> {
        self.devices.get(public_key).map(|d| d.clone())
    }

    pub fn log_event(&mut self, device_id: &String, event: AuthEvent) {
        let entries = self.logs.entry(device_id.clone()).or_default();
        entries.push(event);
        self.persist();
    }

    fn persist(&self) {
        let file = File::create("state_audit.json").unwrap();
        let mut writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self).unwrap();
    }
}

fn load_data() -> Result<DataStore, io::Error> {
    let f = File::open("state.json")?;
    let reader = BufReader::new(f);
    Ok(serde_json::from_reader(reader)?)
}
