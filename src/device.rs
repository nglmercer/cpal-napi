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

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Internal,
    Usb,
    Bluetooth,
    Network,
    Firewire,
    Virtual,
    Other,
}

#[napi(object)]
pub struct DeviceDescription {
    pub name: String,
    pub direction: DeviceDirection,
    pub device_type: DeviceType,
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
    device_type: Option<DeviceType>,
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
            device_type: None,
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
    pub fn device_type(&mut self, device_type: DeviceType) {
        self.device_type = Some(device_type);
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
            device_type: self.device_type.unwrap_or(DeviceType::Other),
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
        let _gag = StderrGag::maybe_gag();
        let name = self.name().unwrap_or_else(|_| "Unknown".to_string());
        let lower_name = name.to_lowercase();

        let device_type = if lower_name.contains("usb") {
            DeviceType::Usb
        } else if lower_name.contains("bluetooth") || lower_name.contains("bluez") {
            DeviceType::Bluetooth
        } else if lower_name.contains("network") {
            DeviceType::Network
        } else if lower_name.contains("virtual")
            || lower_name.contains("pipewire")
            || lower_name.contains("jack")
            || lower_name.contains("discard")
        {
            DeviceType::Virtual
        } else if lower_name.contains("firewire") {
            DeviceType::Firewire
        } else {
            DeviceType::Other
        };

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
        let is_loopback = {
            #[cfg(target_os = "windows")]
            {
                if self.host_id == crate::host::HostId::Wasapi {
                    max_output_channels > 0
                } else {
                    false
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                false
            }
        };

        Ok(DeviceDescription {
            name,
            direction,
            device_type,
            host_id: self.host_id,
            max_input_channels,
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
        Ok(self
            .inner
            .description()
            .map_err(|e| Error::from_reason(format!("Failed to get name: {}", e)))?
            .name()
            .to_string())
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

        // Noise gate threshold - samples below this amplitude are treated as silence
        // This prevents phantom buffer/noise when microphone is muted or silent
        // Set above typical microphone noise floor (~0.008) to filter out hardware noise
        const NOISE_GATE_THRESHOLD: f32 = 0.015;

        macro_rules! build_input {
            ($t:ty) => {
                self.inner.build_input_stream(
                    &cpal_config,
                    move |data: &[$t], _| {
                        // First, check if the entire chunk is silence
                        // This is more efficient than checking each sample individually
                        let has_signal = data.iter().any(|sample| {
                            let value = cpal::Sample::to_sample::<f32>(*sample);
                            value.abs() >= NOISE_GATE_THRESHOLD
                        });

                        // If the entire chunk is silence, don't add anything to the buffer
                        if !has_signal {
                            return;
                        }

                        let mut buffer = shared_buffer.lock().unwrap();
                        for frame in data.chunks(channels) {
                            if let Some(sample) = frame.first() {
                                let value = cpal::Sample::to_sample::<f32>(*sample);
                                // Apply noise gate: if the sample is below threshold, treat as silence
                                let gated_value = if value.abs() < NOISE_GATE_THRESHOLD {
                                    0.0
                                } else {
                                    value
                                };
                                buffer.push_back(gated_value);
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            };
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
