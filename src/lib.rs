pub mod buffer;
pub mod config;
pub mod device;
pub mod error;
pub mod host;
pub mod logger;
pub mod stream;
pub mod types;

pub use buffer::*;
pub use config::*;
pub use device::*;
pub use error::*;
pub use host::*;
pub use logger::{set_debug, set_suppress_alsa_logs};
pub use stream::*;
pub use types::*;

use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub fn get_available_device_names() -> Result<Vec<String>> {
    let host = crate::host::get_default_host();
    let devices = host.devices()?;
    let mut names = Vec::new();
    for d in devices {
        if let Ok(name) = d.name() {
            names.push(name);
        }
    }
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_hosts() {
        let hosts = get_available_host_names();
        assert!(!hosts.is_empty());
    }

    #[test]
    fn test_default_host() {
        let host = get_default_host();
        assert!(!host.name().is_empty());
    }

    #[test]
    fn test_all_hosts() {
        let hosts = get_supported_hosts();
        assert!(!hosts.is_empty());
    }
}
