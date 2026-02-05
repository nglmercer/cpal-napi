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
    fn test_list_device_names() {
        let host = crate::host::get_default_host();
        println!("\n--- Audio Device Diagnostic ---");
        println!("Default Host: {}", host.name());
        if let Ok(devices) = host.devices() {
            println!("Devices found: {}", devices.len());
            for (i, d) in devices.iter().enumerate() {
                let name = d.name().unwrap_or_else(|_| "Unknown".to_string());
                let id = d.id().map(|id| id.id).unwrap_or_else(|_| "Unknown".to_string());
                if let Ok(desc) = d.description() {
                    println!("{}. Name: {}", i + 1, name);
                    println!("   Channels: {} in / {} out", desc.max_input_channels, desc.max_output_channels);
                    println!("   ID: {}", id);
                } else {
                    println!("{}. {} (Description failed)", i + 1, name);
                }
            }
        }
        println!("-------------------------------\n");
    }
}
