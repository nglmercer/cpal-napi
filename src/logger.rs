use std::sync::atomic::{AtomicBool, Ordering};
use napi_derive::napi;

static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);
static SUPPRESS_ALSA_LOGS: AtomicBool = AtomicBool::new(true);

#[napi]
pub fn set_debug(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::SeqCst);
}

pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::SeqCst)
}

/// Enable or disable suppression of ALSA/JACK stderr messages.
/// When enabled (default), noisy ALSA library errors are redirected to /dev/null.
/// Disable this if you need to debug audio driver issues.
#[napi]
pub fn set_suppress_alsa_logs(suppress: bool) {
    SUPPRESS_ALSA_LOGS.store(suppress, Ordering::SeqCst);
}

pub fn should_suppress_alsa_logs() -> bool {
    SUPPRESS_ALSA_LOGS.load(Ordering::SeqCst)
}

pub fn log(msg: &str) {
    if is_debug_enabled() {
        eprintln!("[cpal-napi] {}", msg);
    }
}
