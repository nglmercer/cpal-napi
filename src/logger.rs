use std::sync::atomic::{AtomicBool, Ordering};
use napi_derive::napi;

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

#[napi]
pub fn set_debug(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::SeqCst);
}

pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::SeqCst)
}

pub fn log(msg: &str) {
    if is_debug_enabled() {
        eprintln!("[cpal-napi] {}", msg);
    }
}
