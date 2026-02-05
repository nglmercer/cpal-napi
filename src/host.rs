use crate::device::AudioDevice;
use cpal::traits::HostTrait;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::os::raw::{c_char, c_int};

#[cfg(target_os = "linux")]
extern "C" {
    fn snd_lib_error_set_handler(
        handler: Option<
            unsafe extern "C" fn(*const c_char, c_int, *const c_char, c_int, *const c_char),
        >,
    ) -> c_int;
}

#[cfg(target_os = "linux")]
static GAG_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(target_os = "linux")]
pub struct StderrGag {
    original_fd: i32,
    _guard: std::sync::MutexGuard<'static, ()>,
}

#[cfg(target_os = "linux")]
impl StderrGag {
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
                original_fd,
                _guard: guard,
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
        unsafe {
            libc::dup2(self.original_fd, libc::STDERR_FILENO);
            libc::close(self.original_fd);
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
}

#[cfg(not(target_os = "linux"))]
impl Default for StderrGag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
unsafe extern "C" fn silent_error_handler(
    _file: *const c_char,
    _line: c_int,
    _function: *const c_char,
    _err: c_int,
    _fmt: *const c_char,
) {
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
        let host_id = self.inner.id().into();
        let devices = self
            .inner
            .devices()
            .map_err(|e| Error::from_reason(format!("Failed to get devices: {}", e)))?;
        Ok(devices.map(|d| AudioDevice { inner: d, host_id }).collect())
    }

    #[napi]
    pub fn input_devices(&self) -> Result<Vec<AudioDevice>> {
        let host_id = self.inner.id().into();
        let devices = self
            .inner
            .input_devices()
            .map_err(|e| Error::from_reason(format!("Failed to get input devices: {}", e)))?;
        Ok(devices.map(|d| AudioDevice { inner: d, host_id }).collect())
    }

    #[napi]
    pub fn output_devices(&self) -> Result<Vec<AudioDevice>> {
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
            std::env::set_var("JACK_NO_START_SERVER", "1");
            // Some distributions use these
            std::env::set_var("JACK_START_SERVER", "0");
            // Enable ALSA thread safety to prevent double-frees/corruption
            // This MUST be set before any ALSA function is called.
            std::env::set_var("LIBASOUND_THREAD_SAFE", "1");

            unsafe {
                snd_lib_error_set_handler(Some(silent_error_handler));
            }
        });
    }
}

#[napi]
pub fn get_default_host() -> AudioHost {
    silence_host_logs();
    AudioHost {
        inner: cpal::default_host(),
    }
}

#[napi]
pub fn host_from_id(id: HostId) -> Result<AudioHost> {
    silence_host_logs();
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
    silence_host_logs();
    cpal::available_hosts()
        .iter()
        .map(|h| h.name().to_string())
        .collect()
}

#[napi]
pub fn get_all_hosts() -> Vec<HostId> {
    silence_host_logs();
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
