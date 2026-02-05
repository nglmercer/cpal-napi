use crate::device::AudioDevice;
use cpal::traits::HostTrait;
use napi::bindgen_prelude::*;
use napi_derive::napi;
#[allow(unused_imports)]
use std::os::raw::c_char;

#[cfg(target_os = "linux")]
static GAG_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(target_os = "linux")]
pub struct StderrGag {
    original_fd: Option<i32>,
    _guard: Option<std::sync::MutexGuard<'static, ()>>,
}

#[cfg(target_os = "linux")]
impl StderrGag {
    /// Creates a StderrGag that redirects stderr to /dev/null
    pub fn new() -> Self {
        let guard = GAG_MUTEX.lock().unwrap();
        unsafe {
            let original_fd = libc::dup(libc::STDERR_FILENO);
            let null_file = b"/dev/null\0";
            let null_fd = libc::open(null_file.as_ptr() as *const c_char, libc::O_WRONLY);
            if null_fd != -1 {
                libc::dup2(null_fd, libc::STDERR_FILENO);
                libc::close(null_fd);
            }
            StderrGag {
                original_fd: Some(original_fd),
                _guard: Some(guard),
            }
        }
    }

    /// Creates a StderrGag only if suppression is enabled in logger
    pub fn maybe_gag() -> Self {
        if crate::logger::should_suppress_alsa_logs() {
            Self::new()
        } else {
            StderrGag {
                original_fd: None,
                _guard: None,
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Default for StderrGag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
impl Drop for StderrGag {
    fn drop(&mut self) {
        if let Some(fd) = self.original_fd {
            unsafe {
                libc::dup2(fd, libc::STDERR_FILENO);
                libc::close(fd);
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub struct StderrGag;

#[cfg(not(target_os = "linux"))]
impl StderrGag {
    pub fn new() -> Self {
        StderrGag
    }

    pub fn maybe_gag() -> Self {
        StderrGag
    }
}

#[cfg(not(target_os = "linux"))]
impl Default for StderrGag {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostId {
    Alsa,
    Jack,
    Wasapi,
    Asio,
    CoreAudio,
    Emscripten,
    Other,
}

impl From<cpal::HostId> for HostId {
    fn from(id: cpal::HostId) -> Self {
        match id.name().to_lowercase().as_str() {
            "alsa" => HostId::Alsa,
            "jack" => HostId::Jack,
            "wasapi" => HostId::Wasapi,
            "asio" => HostId::Asio,
            "coreaudio" => HostId::CoreAudio,
            "emscripten" => HostId::Emscripten,
            _ => HostId::Other,
        }
    }
}

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
        let _gag = StderrGag::maybe_gag();
        let host_id = self.inner.id().into();
        let devices = self
            .inner
            .devices()
            .map_err(|e| Error::from_reason(format!("Failed to get devices: {}", e)))?;
        Ok(devices.map(|d| AudioDevice { inner: d, host_id }).collect())
    }

    #[napi]
    pub fn input_devices(&self) -> Result<Vec<AudioDevice>> {
        let _gag = StderrGag::maybe_gag();
        let host_id = self.inner.id().into();
        let devices = self
            .inner
            .input_devices()
            .map_err(|e| Error::from_reason(format!("Failed to get input devices: {}", e)))?;
        Ok(devices.map(|d| AudioDevice { inner: d, host_id }).collect())
    }

    #[napi]
    pub fn output_devices(&self) -> Result<Vec<AudioDevice>> {
        let _gag = StderrGag::maybe_gag();
        let host_id = self.inner.id().into();
        let devices = self
            .inner
            .output_devices()
            .map_err(|e| Error::from_reason(format!("Failed to get output devices: {}", e)))?;
        Ok(devices.map(|d| AudioDevice { inner: d, host_id }).collect())
    }

    #[napi]
    pub fn default_input_device(&self) -> Option<AudioDevice> {
        let host_id = self.inner.id().into();
        self.inner
            .default_input_device()
            .map(|d| AudioDevice { inner: d, host_id })
    }

    #[napi]
    pub fn default_output_device(&self) -> Option<AudioDevice> {
        let host_id = self.inner.id().into();
        self.inner
            .default_output_device()
            .map(|d| AudioDevice { inner: d, host_id })
    }
}

#[napi]
pub fn silence_host_logs() {
    #[cfg(target_os = "linux")]
    {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            // Prevent JACK from trying to start a server when probing
            // We use libc::setenv if we can, but std::env::set_var is what Rust provides.
            // We do this once at the very beginning.
            std::env::set_var("JACK_NO_START_SERVER", "1");
            std::env::set_var("JACK_START_SERVER", "0");
            std::env::set_var("LIBASOUND_THREAD_SAFE", "1");
        });
    }
}

#[cfg(target_os = "linux")]
#[ctor::ctor]
fn init() {
    // Call it early to set env vars before ALSA/JACK are initialized
    silence_host_logs();
}

#[napi]
pub fn get_default_host() -> AudioHost {
    AudioHost {
        inner: cpal::default_host(),
    }
}

#[napi]
pub fn host_from_id(id: HostId) -> Result<AudioHost> {
    let cpal_id = match id {
        #[cfg(target_os = "linux")]
        HostId::Alsa => Some(cpal::HostId::Alsa),
        #[cfg(target_os = "macos")]
        HostId::CoreAudio => Some(cpal::HostId::CoreAudio),
        #[cfg(target_os = "windows")]
        HostId::Wasapi => Some(cpal::HostId::Wasapi),
        #[cfg(all(target_os = "windows", feature = "asio"))]
        HostId::Asio => Some(cpal::HostId::Asio),
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
            HostId::Alsa => "alsa",
            HostId::Jack => "jack",
            HostId::Wasapi => "wasapi",
            HostId::Asio => "asio",
            HostId::CoreAudio => "coreaudio",
            HostId::Emscripten => "emscripten",
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
pub fn get_all_hosts() -> Vec<HostId> {
    vec![
        HostId::Alsa,
        HostId::Jack,
        HostId::Wasapi,
        HostId::Asio,
        HostId::CoreAudio,
        HostId::Emscripten,
    ]
}

#[napi]
pub fn get_all_hosts_list() -> Vec<HostId> {
    get_all_hosts()
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
