pub mod error;
pub mod serialize;
pub mod checks;
pub mod log;
pub mod audit;
pub mod bls;

pub mod time {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }
}
