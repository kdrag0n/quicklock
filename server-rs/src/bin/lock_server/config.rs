use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use once_cell::sync::{Lazy};
use serde::{Serialize, Deserialize};

fn time_grace_period() -> u64 { 5 * 60 * 1000 } // 5 min
fn relock_delay() -> u64 { 3 * 1000 } // 3 sec
fn require_audit() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub ha_api_key: String,
    pub entities: HashMap<String, Entity>,

    #[serde(default = "time_grace_period")]
    pub time_grace_period: u64,
    #[serde(default = "relock_delay")]
    pub relock_delay: u64,
    #[serde(default = "require_audit")]
    pub require_audit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub ha_entity: String,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::load().unwrap());

impl Config {
    fn load() -> anyhow::Result<Config> {
        let f = File::open("config.json")?;
        let reader = BufReader::new(f);
        let config: Config = serde_json::from_reader(reader)?;
        Ok(config)
    }
}
