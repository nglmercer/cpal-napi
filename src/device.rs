use crate::buffer::AudioBuffer;
use crate::config::{StreamConfig, SupportedStreamConfig};
use crate::host::StderrGag;
use crate::stream::AudioStream;
use cpal::traits::DeviceTrait;
use napi::bindgen_prelude::*;

use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi_derive::napi;

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceDirection {
    Input,
    Output,
    Both,
    None,
}

#[napi(object)]
pub struct DeviceDescription {
    pub name: String,
    pub direction: DeviceDirection,
    pub host_id: crate::host::HostId,
    pub max_input_channels: u16,
    pub max_output_channels: u16,
    pub available: bool,
    pub is_loopback: bool,
}

#[napi]
pub struct DeviceDescriptionBuilder {
    name: Option<String>,
    direction: Option<DeviceDirection>,
    host_id: Option<crate::host::HostId>,
}

impl Default for DeviceDescriptionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl DeviceDescriptionBuilder {
    #[napi(constructor)]
    pub fn new() -> DeviceDescriptionBuilder {
        DeviceDescriptionBuilder {
            name: None,
            direction: None,
            host_id: None,
        }
    }

    #[napi]
    pub fn name(&mut self, name: String) {
        self.name = Some(name);
    }

    #[napi]
    pub fn direction(&mut self, direction: DeviceDirection) {
        self.direction = Some(direction);
    }

    #[napi]
    pub fn host_id(&mut self, host_id: crate::host::HostId) {
        self.host_id = Some(host_id);
    }

    #[napi]
    pub fn build(&self) -> DeviceDescription {
        DeviceDescription {
            name: self.name.clone().unwrap_or_default(),
            direction: self.direction.unwrap_or(DeviceDirection::None),
            host_id: self.host_id.unwrap_or(crate::host::HostId::Other),
            max_input_channels: 0,
            max_output_channels: 0,
            available: false,
            is_loopback: false,
        }
    }
}

#[napi]
pub struct AudioDevice {
    pub(crate) inner: cpal::Device,
    pub(crate) host_id: crate::host::HostId,
}

#[napi(object)]
pub struct DeviceId {
    pub id: String,
}

#[napi(object)]
pub struct Data {
    pub sample_format: crate::types::SampleFormat,
}

#[napi]
impl AudioDevice {
    #[napi]
    pub fn id(&self) -> Result<DeviceId> {
        Ok(DeviceId {
            id: self
                .inner
                .id()
                .map_err(|e| Error::from_reason(format!("Failed to get device id: {}", e)))?
                .to_string(),
        })
    }

    #[napi]
    pub fn description(&self) -> Result<DeviceDescription> {
        let mut name = "Unknown".to_string();
        let mut is_loopback_prop = false;

        #[cfg(target_os = "windows")]
        if self.host_id == crate::host::HostId::Wasapi {
            if let Ok(id_obj) = self.id() {
                if let Some((better, lp)) = get_windows_friendly_name(&id_obj.id) {
                    name = better;
                    is_loopback_prop = lp;
                }
            }
        }

        if name == "Unknown" {
            name = self.name().unwrap_or_else(|_| "Unknown".to_string());
        }

        let max_input_channels = self
            .inner
            .supported_input_configs()
            .map(|configs| configs.map(|c| c.channels()).max().unwrap_or(0))
            .unwrap_or(0);

        let max_output_channels = self
            .inner
            .supported_output_configs()
            .map(|configs| configs.map(|c| c.channels()).max().unwrap_or(0))
            .unwrap_or(0);

        let direction = match (max_input_channels > 0, max_output_channels > 0) {
            (true, true) => DeviceDirection::Both,
            (true, false) => DeviceDirection::Input,
            (false, true) => DeviceDirection::Output,
            (false, false) => DeviceDirection::None,
        };
        let available = max_input_channels > 0 || max_output_channels > 0;

        // Final loopback decision
        let is_loopback = {
            #[cfg(target_os = "windows")]
            {
                if self.host_id == crate::host::HostId::Wasapi {
                    // Use property-based detection primarily
                    is_loopback_prop || name.to_lowercase().contains("loopback")
                } else {
                    false
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                false
            }
        };

        // If it's a loopback on Windows, it might not report input channels yet,
        // but it will support them (matching output channels).
        let effective_max_input = if is_loopback && max_input_channels == 0 {
            max_output_channels.max(2)
        } else {
            max_input_channels
        };

        Ok(DeviceDescription {
            name,
            direction,
            host_id: self.host_id,
            max_input_channels: effective_max_input,
            max_output_channels,
            available,
            is_loopback,
        })
    }

    #[napi]
    pub fn is_available(&self) -> bool {
        let _gag = StderrGag::maybe_gag();
        let max_input_channels = self
            .inner
            .supported_input_configs()
            .map(|configs| configs.map(|c| c.channels()).max().unwrap_or(0))
            .unwrap_or(0);

        let max_output_channels = self
            .inner
            .supported_output_configs()
            .map(|configs| configs.map(|c| c.channels()).max().unwrap_or(0))
            .unwrap_or(0);

        max_input_channels > 0 || max_output_channels > 0
    }

    #[napi]
    pub fn is_input(&self) -> bool {
        let _gag = StderrGag::maybe_gag();
        self.inner
            .supported_input_configs()
            .map(|mut i| i.next().is_some())
            .unwrap_or(false)
    }

    #[napi]
    pub fn is_output(&self) -> bool {
        let _gag = StderrGag::maybe_gag();
        self.inner
            .supported_output_configs()
            .map(|mut i| i.next().is_some())
            .unwrap_or(false)
    }

    #[napi]
    pub fn name(&self) -> Result<String> {
        let name = self
            .inner
            .description()
            .map_err(|e| Error::from_reason(format!("Failed to get name: {}", e)))?
            .name()
            .to_string();

        #[cfg(target_os = "windows")]
        {
            if self.host_id == crate::host::HostId::Wasapi {
                if let Ok(id_obj) = self.id() {
                    let cpal_name = self
                        .inner
                        .description()
                        .map(|d| d.name().to_string())
                        .unwrap_or_default();
                    if let Some((better, _)) = get_windows_friendly_name(&id_obj.id) {
                        // Preserve the "(Loopback)" suffix if CPAL added it
                        if cpal_name.to_lowercase().contains("loopback")
                            && !better.to_lowercase().contains("loopback")
                        {
                            return Ok(format!("{} (Loopback)", better));
                        }
                        return Ok(better);
                    }
                }
            }
        }

        Ok(name)
    }

    #[napi]
    pub fn default_output_config(&self) -> Result<StreamConfig> {
        let config = self.inner.default_output_config().map_err(|e| {
            Error::from_reason(format!("Failed to get default output config: {}", e))
        })?;
        Ok(config.into())
    }

    #[napi]
    pub fn default_input_config(&self) -> Result<StreamConfig> {
        let config = self.inner.default_input_config().map_err(|e| {
            Error::from_reason(format!("Failed to get default input config: {}", e))
        })?;
        Ok(config.into())
    }

    #[napi]
    pub fn supported_output_configs(&self) -> Result<Vec<SupportedStreamConfig>> {
        let _gag = StderrGag::maybe_gag();
        let configs = self.inner.supported_output_configs().map_err(|e| {
            Error::from_reason(format!("Failed to get supported output configs: {}", e))
        })?;
        Ok(configs.map(|c| c.into()).collect())
    }

    #[napi]
    pub fn supported_input_configs(&self) -> Result<Vec<SupportedStreamConfig>> {
        let _gag = StderrGag::maybe_gag();
        let configs = self.inner.supported_input_configs().map_err(|e| {
            Error::from_reason(format!("Failed to get supported input configs: {}", e))
        })?;
        Ok(configs.map(|c| c.into()).collect())
    }

    #[napi]
    pub fn create_output_stream(
        &self,
        config: StreamConfig,
        buffer: &AudioBuffer,
        #[napi(ts_arg_type = "(err: string) => void")] error_callback: Option<
            Function<(String,), ()>,
        >,
    ) -> Result<AudioStream> {
        let cpal_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: config.sample_rate,
            buffer_size: config.buffer_size.into(),
        };

        let channels = config.channels as usize;
        let shared_buffer = buffer.inner.clone();

        let tsfn = if let Some(cb) = error_callback {
            let f = cb
                .build_threadsafe_function::<String>()
                .callee_handled::<true>()
                .build_callback(|ctx| Ok((ctx.value.clone(),)))?;
            Some(f)
        } else {
            None
        };

        let err_fn = move |err: cpal::StreamError| {
            let msg = err.to_string();
            if let Some(ref f) = tsfn {
                let _ = f.call(Ok(msg.clone()), ThreadsafeFunctionCallMode::NonBlocking);
            }
            if !msg.contains("underrun") && !msg.contains("overrun") {
                crate::logger::log(&format!("an error occurred on stream: {}", err));
            }
        };

        let sample_format = config.sample_format.into();

        macro_rules! build_output {
            ($t:ty) => {
                self.inner.build_output_stream(
                    &cpal_config,
                    move |data: &mut [$t], _| {
                        let mut buffer = shared_buffer.lock().unwrap();
                        for frame in data.chunks_mut(channels) {
                            let value = buffer.pop_front().unwrap_or(0.0);
                            let sample = cpal::Sample::from_sample(value);
                            for s in frame.iter_mut() {
                                *s = sample;
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            };
        }

        let stream = match sample_format {
            cpal::SampleFormat::I8 => build_output!(i8),
            cpal::SampleFormat::U8 => build_output!(u8),
            cpal::SampleFormat::I16 => build_output!(i16),
            cpal::SampleFormat::U16 => build_output!(u16),
            cpal::SampleFormat::I32 => build_output!(i32),
            cpal::SampleFormat::U32 => build_output!(u32),
            cpal::SampleFormat::F32 => build_output!(f32),
            _ => {
                return Err(Error::from_reason(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )))
            }
        }
        .map_err(|e| {
            let name = self.name().unwrap_or_else(|_| "Unknown".to_string());
            Error::from_reason(format!(
                "Failed to build output stream on '{}': {}",
                name, e
            ))
        })?;

        Ok(AudioStream::new(stream))
    }

    #[napi]
    pub fn create_input_stream(
        &self,
        config: StreamConfig,
        buffer: &AudioBuffer,
        #[napi(ts_arg_type = "(err: string) => void")] error_callback: Option<
            Function<(String,), ()>,
        >,
    ) -> Result<AudioStream> {
        let cpal_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: config.sample_rate,
            buffer_size: config.buffer_size.into(),
        };

        let channels = config.channels as usize;
        let shared_buffer = buffer.inner.clone();

        let tsfn = if let Some(cb) = error_callback {
            let f = cb
                .build_threadsafe_function::<String>()
                .callee_handled::<true>()
                .build_callback(|ctx| Ok((ctx.value.clone(),)))?;
            Some(std::sync::Arc::new(f))
        } else {
            None
        };

        // Note: err_fn definition moved inside macro to avoid 'use of moved value' errors across match arms.

        let sample_format = config.sample_format.into();
        let noise_gate = config.noise_gate_threshold.map(|t| t as f32);
        let mix_mode = config
            .mix_mode
            .unwrap_or(crate::config::ChannelMixMode::Auto);

        macro_rules! build_input {
            ($t:ty) => {{
                let mut peak_l = 0.0f32;
                let mut peak_r = 0.0f32;
                let alpha = 0.95f32; // Env decay

                // Prepare dependencies for this specific closure instance
                let tsfn_local = tsfn.clone();
                let shared_buffer_local = shared_buffer.clone();

                // If mix_mode is Copy (Enum), this is a copy. If not, we might need a clone.
                // Assuming Copy for now based on typical usage, but shadowing ensures we have a local binding.
                let mix_mode_local = mix_mode;

                let err_fn_local = move |err: cpal::StreamError| {
                    let msg = err.to_string();
                    if let Some(ref f) = tsfn_local {
                        let _ = f.call(Ok(msg.clone()), ThreadsafeFunctionCallMode::NonBlocking);
                    }
                    if !msg.contains("underrun") && !msg.contains("overrun") {
                        crate::logger::log(&format!("an error occurred on stream: {}", err));
                    }
                };

                self.inner.build_input_stream(
                    &cpal_config,
                    move |data: &[$t], _| {
                        // 1. Local buffer to process samples before locking.
                        let mut processed = Vec::with_capacity(data.len() / channels);

                        match channels {
                            1 => {
                                // Pure mono
                                processed.extend(data.iter().map(|&s| cpal::Sample::to_sample::<f32>(s)));
                            },
                            2 => {
                                match mix_mode_local {
                                    crate::config::ChannelMixMode::Left => {
                                        for frame in data.chunks(2) {
                                            processed.push(cpal::Sample::to_sample::<f32>(frame[0]));
                                        }
                                    },
                                    crate::config::ChannelMixMode::Right => {
                                        for frame in data.chunks(2) {
                                            processed.push(cpal::Sample::to_sample::<f32>(frame[1]));
                                        }
                                    },
                                    crate::config::ChannelMixMode::Balanced => {
                                        for frame in data.chunks(2) {
                                            let c0 = cpal::Sample::to_sample::<f32>(frame[0]);
                                            let c1 = cpal::Sample::to_sample::<f32>(frame[1]);
                                            processed.push((c0 + c1) * 0.5);
                                        }
                                    },
                                    crate::config::ChannelMixMode::Auto => {
                                        // 1. Analyze the buffer (Per-Buffer Analysis)
                                        let mut max_l = 0.0f32;
                                        let mut max_r = 0.0f32;
                                        for frame in data.chunks(2) {
                                            let l = cpal::Sample::to_sample::<f32>(frame[0]).abs();
                                            let r = cpal::Sample::to_sample::<f32>(frame[1]).abs();
                                            if l > max_l { max_l = l; }
                                            if r > max_r { max_r = r; }
                                        }

                                        // 2. Update Envelopes
                                        peak_l = max_l.max(peak_l * alpha);
                                        peak_r = max_r.max(peak_r * alpha);

                                        // 3. Decide Strategy
                                        let threshold = noise_gate.unwrap_or(0.0);
                                        let l_active = peak_l >= threshold;
                                        let r_active = peak_r >= threshold;

                                        let use_l;
                                        let use_r;

                                        if l_active && !r_active {
                                            use_l = true;
                                            use_r = false;
                                        } else if r_active && !l_active {
                                            use_l = false;
                                            use_r = true;
                                        } else {
                                            // Both active/inactive: compare smoothed peaks
                                            use_l = peak_l > peak_r * 1.15;
                                            use_r = peak_r > peak_l * 1.15;
                                        }

                                        if use_l {
                                            for frame in data.chunks(2) {
                                                processed.push(cpal::Sample::to_sample::<f32>(frame[0]));
                                            }
                                        } else if use_r {
                                            for frame in data.chunks(2) {
                                                processed.push(cpal::Sample::to_sample::<f32>(frame[1]));
                                            }
                                        } else {
                                            // Balanced fallback
                                            for frame in data.chunks(2) {
                                                let c0 = cpal::Sample::to_sample::<f32>(frame[0]);
                                                let c1 = cpal::Sample::to_sample::<f32>(frame[1]);
                                                processed.push((c0 + c1) * 0.5);
                                            }
                                        }
                                    }
                                }
                            },
                            _ => {
                                // Multi-channel fallback
                                for frame in data.chunks(channels) {
                                    let sum: f64 = frame.iter().map(|&s| cpal::Sample::to_sample::<f32>(s) as f64).sum();
                                    processed.push((sum / channels as f64) as f32);
                                }
                            }
                        }

                        // 2. Gate
                        if let Some(threshold) = noise_gate {
                            for sample in processed.iter_mut() {
                                if sample.abs() < threshold {
                                    *sample = 0.0;
                                }
                            }
                        }

                        // 3. Push
                        if let Ok(mut buffer) = shared_buffer_local.lock() {
                            buffer.extend(processed);

                            if buffer.len() > 44100 {
                                let to_remove = buffer.len() - 22050; // Keep ~0.5s
                                drop(buffer.drain(0..to_remove));
                            }
                        }
                    },
                    err_fn_local,
                    None,
                )
            }};
        }

        let stream = match sample_format {
            cpal::SampleFormat::I8 => build_input!(i8),
            cpal::SampleFormat::U8 => build_input!(u8),
            cpal::SampleFormat::I16 => build_input!(i16),
            cpal::SampleFormat::U16 => build_input!(u16),
            cpal::SampleFormat::I32 => build_input!(i32),
            cpal::SampleFormat::U32 => build_input!(u32),
            cpal::SampleFormat::F32 => build_input!(f32),
            _ => {
                return Err(Error::from_reason(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )))
            }
        }
        .map_err(|e| {
            let name = self.name().unwrap_or_else(|_| "Unknown".to_string());
            Error::from_reason(format!("Failed to build input stream on '{}': {}", name, e))
        })?;

        Ok(AudioStream::new(stream))
    }
}

#[cfg(target_os = "windows")]
fn get_windows_friendly_name(device_id: &str) -> Option<(String, bool)> {
    use windows::core::Interface;
    use windows::Win32::Devices::FunctionDiscovery::{
        PKEY_DeviceInterface_FriendlyName, PKEY_Device_DeviceDesc, PKEY_Device_FriendlyName,
    };
    use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
    use windows::Win32::Media::Audio::{
        eRender, IMMDevice, IMMDeviceEnumerator, IMMEndpoint, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
    };
    use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PROPERTYKEY};

    unsafe {
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() && hr != RPC_E_CHANGED_MODE {
            return None;
        }

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;

        // CPAL prepends host name to ID, e.g. "wasapi:{...}". Strip it for native API.
        let native_id = device_id.strip_prefix("wasapi:").unwrap_or(device_id);

        let device_id_h = windows::core::HSTRING::from(native_id);
        let device: IMMDevice = enumerator.GetDevice(&device_id_h).ok()?;

        // Detect if this is a render endpoint (eRender) or capture (eCapture)
        // If we are looking at this device in the context of an "Input" list but it is eRender,
        // then it is a WASAPI loopback device.
        let is_loopback = (|| {
            let endpoint: IMMEndpoint = device.cast().ok()?;
            let flow = endpoint.GetDataFlow().ok()?;
            // eRender = 0. This flow on a "Capture" context means Loopback.
            Some(flow == eRender)
        })()
        .unwrap_or(false);

        let store: IPropertyStore = device.OpenPropertyStore(STGM_READ).ok()?;

        let get_prop = |key: &PROPERTYKEY| -> Option<String> {
            if let Ok(prop) = store.GetValue(key) {
                if let Ok(ptr) = PropVariantToStringAlloc(&prop) {
                    let s = ptr.to_string().ok().filter(|s| !s.trim().is_empty());
                    windows::Win32::System::Com::CoTaskMemFree(Some(ptr.0 as _));
                    return s;
                }
            }
            None
        };

        // Standard Windows Audio properties
        let endpoint_name = get_prop(&PKEY_Device_FriendlyName); // e.g., "Microphone" or "Altavoces"
        let hardware_name = get_prop(&PKEY_Device_DeviceDesc); // e.g., "High Definition Audio Device"
        let interface_name = get_prop(&PKEY_DeviceInterface_FriendlyName);

        // DEVPKEY_Device_Controller_FriendlyName: {b3f8fa76-5d97-4876-8f2d-052e35b1c906}, 2
        let controller_name = {
            let key = PROPERTYKEY {
                fmtid: windows::core::GUID::from_u128(0xb3f8fa76_5d97_4876_8f2d_052e35b1c906),
                pid: 2,
            };
            get_prop(&key)
        };

        // Attempt to find a more specific hardware name
        let mut best_hardware = hardware_name;

        // DEVPKEY_Device_BusReportedDeviceDescription: {540b947e-8b40-45bc-a8a2-6a0b894cbda2}, 4
        let bus_desc = {
            let key = PROPERTYKEY {
                fmtid: windows::core::GUID::from_u128(0x540b947e_8b40_45bc_a8a2_6a0b894cbda2),
                pid: 4,
            };
            get_prop(&key)
        };

        for n in [bus_desc, controller_name].into_iter().flatten() {
            if let Some(ref current) = best_hardware {
                if n.len() > current.len() {
                    best_hardware = Some(n);
                }
            } else {
                best_hardware = Some(n);
            }
        }

        let final_name = match (endpoint_name, best_hardware) {
            (Some(e), Some(h)) => {
                let e_lower = e.to_lowercase();
                let h_lower = h.to_lowercase();

                if e_lower == h_lower || e_lower.contains(&h_lower) {
                    Some(e)
                } else if h_lower.contains(&e_lower) {
                    Some(h)
                } else {
                    // Combine them if they are distinct
                    Some(format!("{} ({})", e, h))
                }
            }
            (Some(e), None) => Some(e),
            (None, Some(h)) => Some(h),
            (None, None) => interface_name,
        }?;

        Some((final_name, is_loopback))
    }
}
