use crate::buffer::AudioBuffer;
use crate::config::{BufferSize, StreamConfig, SupportedStreamConfig};
use crate::stream::AudioStream;
use cpal::traits::DeviceTrait;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceDirection {
    Input,
    Output,
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

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceType {
    Alsa,
    Jack,
    Wasapi,
    Asio,
    CoreAudio,
    Emscripten,
    Other,
}

#[napi(object)]
pub struct DeviceDescription {
    pub name: String,
    pub direction: DeviceDirection,
    pub device_type: DeviceType,
    pub interface_type: InterfaceType,
}

#[napi]
pub struct DeviceDescriptionBuilder {
    name: Option<String>,
    direction: Option<DeviceDirection>,
    device_type: Option<DeviceType>,
    interface_type: Option<InterfaceType>,
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
            interface_type: None,
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
    pub fn interface_type(&mut self, interface_type: InterfaceType) {
        self.interface_type = Some(interface_type);
    }

    #[napi]
    pub fn build(&self) -> DeviceDescription {
        DeviceDescription {
            name: self.name.clone().unwrap_or_default(),
            direction: self.direction.unwrap_or(DeviceDirection::Output),
            device_type: self.device_type.unwrap_or(DeviceType::Other),
            interface_type: self.interface_type.unwrap_or(InterfaceType::Other),
        }
    }
}

#[napi]
pub struct AudioDevice {
    pub(crate) inner: cpal::Device,
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
        let desc = self
            .inner
            .description()
            .map_err(|e| Error::from_reason(format!("Failed to get device description: {}", e)))?;
        Ok(DeviceDescription {
            name: desc.name().to_string(),
            direction: match desc.direction() {
                cpal::DeviceDirection::Input => DeviceDirection::Input,
                cpal::DeviceDirection::Output => DeviceDirection::Output,
                _ => DeviceDirection::Output,
            },
            device_type: DeviceType::Other,
            interface_type: InterfaceType::Other,
        })
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
        Ok(StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: BufferSize::Default,
        })
    }

    #[napi]
    pub fn default_input_config(&self) -> Result<StreamConfig> {
        let config = self.inner.default_input_config().map_err(|e| {
            Error::from_reason(format!("Failed to get default input config: {}", e))
        })?;
        Ok(StreamConfig {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            buffer_size: BufferSize::Default,
        })
    }

    #[napi]
    pub fn supported_output_configs(&self) -> Result<Vec<SupportedStreamConfig>> {
        let configs = self.inner.supported_output_configs().map_err(|e| {
            Error::from_reason(format!("Failed to get supported output configs: {}", e))
        })?;
        Ok(configs.map(|c| c.into()).collect())
    }

    #[napi]
    pub fn supported_input_configs(&self) -> Result<Vec<SupportedStreamConfig>> {
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
    ) -> Result<AudioStream> {
        let cpal_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: config.sample_rate,
            buffer_size: config.buffer_size.into(),
        };

        let channels = config.channels as usize;
        let shared_buffer = buffer.inner.clone();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = self
            .inner
            .build_output_stream(
                &cpal_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        let value = buffer.pop_front().unwrap_or(0.0);
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| Error::from_reason(format!("Failed to build stream: {}", e)))?;

        Ok(AudioStream::new(stream))
    }

    #[napi]
    pub fn create_input_stream(
        &self,
        config: StreamConfig,
        buffer: &AudioBuffer,
    ) -> Result<AudioStream> {
        let cpal_config = cpal::StreamConfig {
            channels: config.channels,
            sample_rate: config.sample_rate,
            buffer_size: config.buffer_size.into(),
        };

        let channels = config.channels as usize;
        let shared_buffer = buffer.inner.clone();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = self
            .inner
            .build_input_stream(
                &cpal_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks(channels) {
                        if let Some(sample) = frame.first() {
                            buffer.push_back(*sample);
                        }
                    }
                },
                err_fn,
                None,
            )
            .map_err(|e| Error::from_reason(format!("Failed to build input stream: {}", e)))?;

        Ok(AudioStream::new(stream))
    }
}
