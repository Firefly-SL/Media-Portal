use std::sync::atomic::AtomicBool;

pub mod config;
pub mod utils;

pub static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);
