use crate::device::AudioDevice;
use cpal::traits::HostTrait;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct AudioHost {
    pub(crate) inner: cpal::Host,
}

#[napi]
impl AudioHost {
    #[napi]
    pub fn name(&self) -> String {
        self.inner.id().name().to_string()
    }

    #[napi]
    pub fn devices(&self) -> Result<Vec<AudioDevice>> {
        let devices = self
            .inner
            .devices()
            .map_err(|e| Error::from_reason(format!("Failed to get devices: {}", e)))?;
        Ok(devices.map(|d| AudioDevice { inner: d }).collect())
    }

    #[napi]
    pub fn default_input_device(&self) -> Option<AudioDevice> {
        self.inner
            .default_input_device()
            .map(|d| AudioDevice { inner: d })
    }

    #[napi]
    pub fn default_output_device(&self) -> Option<AudioDevice> {
        self.inner
            .default_output_device()
            .map(|d| AudioDevice { inner: d })
    }
}

#[napi]
pub fn get_default_host() -> AudioHost {
    AudioHost {
        inner: cpal::default_host(),
    }
}

#[napi]
pub fn host_from_id(id: crate::types::HostId) -> Result<AudioHost> {
    let cpal_id = match id {
        #[cfg(target_os = "linux")]
        crate::types::HostId::Alsa => Some(cpal::HostId::Alsa),
        #[cfg(target_os = "macos")]
        crate::types::HostId::CoreAudio => Some(cpal::HostId::CoreAudio),
        #[cfg(target_os = "windows")]
        crate::types::HostId::Wasapi => Some(cpal::HostId::Wasapi),
        #[cfg(all(target_os = "windows", feature = "asio"))]
        crate::types::HostId::Asio => Some(cpal::HostId::Asio),
        // Jack and others are tricky due to features, but we can try to use cpal's available hosts
        _ => None,
    };

    if let Some(cid) = cpal_id {
        cpal::host_from_id(cid)
            .map(|h| AudioHost { inner: h })
            .map_err(|e| Error::from_reason(format!("Failed to initialize host: {}", e)))
    } else {
        // Fallback for cases where we can't name the ID directly but it might be available
        let name = match id {
            crate::types::HostId::Alsa => "alsa",
            crate::types::HostId::Jack => "jack",
            crate::types::HostId::Wasapi => "wasapi",
            crate::types::HostId::Asio => "asio",
            crate::types::HostId::CoreAudio => "coreaudio",
            crate::types::HostId::Emscripten => "emscripten",
            _ => "",
        };

        cpal::available_hosts()
            .iter()
            .find(|h| h.name().to_lowercase() == name)
            .and_then(|h| cpal::host_from_id(*h).ok())
            .map(|h| AudioHost { inner: h })
            .ok_or_else(|| Error::from_reason(format!("Host not available: {:?}", id)))
    }
}

#[napi]
pub fn available_hosts() -> Vec<String> {
    cpal::available_hosts()
        .iter()
        .map(|h| h.name().to_string())
        .collect()
}

#[napi]
pub fn get_all_hosts_list() -> Vec<crate::types::HostId> {
    vec![
        crate::types::HostId::Alsa,
        crate::types::HostId::Jack,
        crate::types::HostId::Wasapi,
        crate::types::HostId::Asio,
        crate::types::HostId::CoreAudio,
        crate::types::HostId::Emscripten,
    ]
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_available_hosts() {
        let hosts = available_hosts();
        assert!(!hosts.is_empty());
    }

    #[test]
    fn test_host_from_id() {
        let hosts = get_all_hosts_list();
        for id in hosts {
            // This might fail if the host is not available on the current platform,
            // but we can at least try to see if it doesn't panic.
            let _ = host_from_id(id);
        }
    }
}
