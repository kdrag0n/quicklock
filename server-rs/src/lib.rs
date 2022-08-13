pub mod error;
pub mod serialize;
pub mod checks;
pub mod log;
pub mod lock;
pub mod audit;
pub mod bls;
pub mod envelope;

pub mod time {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }
}

#[macro_export]
macro_rules! profile {
    ($tag:expr, $code:block) => {
        {
            use std::time::Instant;
            use tracing::log::info;

            info!("[{}] START", $tag);
            let start = Instant::now();
            let ret = $code;
            let end = Instant::now();
            info!("[{}] END: {} ms", $tag, end.duration_since(start).as_millis());
            ret
        }
    };
}
