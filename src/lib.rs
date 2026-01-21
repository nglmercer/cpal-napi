pub mod buffer;
pub mod config;
pub mod device;
pub mod device_description;
pub mod error;
pub mod host;
pub mod stream;
pub mod types;

pub use buffer::*;
pub use config::*;
pub use device::*;
pub use device_description::*;
pub use error::*;
pub use host::*;
pub use stream::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_hosts() {
        let hosts = available_hosts();
        assert!(!hosts.is_empty());
    }

    #[test]
    fn test_default_host() {
        let host = get_default_host();
        assert!(!host.name().is_empty());
    }
}
