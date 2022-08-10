use android_logger::Config;
use log::Level;
use rifgen::rifgen_attr::*;

mod java_glue;
pub use crate::java_glue::*;

pub struct RustLog;

impl RustLog {
    #[generate_interface]
    pub fn init() {
        android_logger::init_once(
            Config::default()
                .with_min_level(Level::Trace)
                .with_tag("RustLib"),
        );
        log_panics::init();
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
