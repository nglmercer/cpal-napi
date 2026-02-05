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
        let name = self.name().unwrap_or_else(|_| "Unknown".to_string());
        
        Ok(DeviceDescription {
            name,
            direction: if self.is_input() {
                DeviceDirection::Input
            } else {
                DeviceDirection::Output
            },
            device_type: DeviceType::Other,
            interface_type: InterfaceType::Other,
        })
    }

    #[napi]
    pub fn is_input(&self) -> bool {
        self.inner.supported_input_configs().map(|mut i| i.next().is_some()).unwrap_or(false)
    }

    #[napi]
    pub fn is_output(&self) -> bool {
        self.inner.supported_output_configs().map(|mut i| i.next().is_some()).unwrap_or(false)
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
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: config.buffer_size.into(),
        };

        let channels = config.channels as usize;
        let shared_buffer = buffer.inner.clone();
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let sample_format = config.sample_format.into();

        let stream = match sample_format {
            cpal::SampleFormat::I8 => self.inner.build_output_stream(
                &cpal_config,
                move |data: &mut [i8], _| {
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
            ),
            cpal::SampleFormat::U8 => self.inner.build_output_stream(
                &cpal_config,
                move |data: &mut [u8], _| {
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
            ),
            cpal::SampleFormat::I16 => self.inner.build_output_stream(
                &cpal_config,
                move |data: &mut [i16], _| {
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
            ),
            cpal::SampleFormat::U16 => self.inner.build_output_stream(
                &cpal_config,
                move |data: &mut [u16], _| {
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
            ),
            cpal::SampleFormat::I32 => self.inner.build_output_stream(
                &cpal_config,
                move |data: &mut [i32], _| {
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
            ),
            cpal::SampleFormat::U32 => self.inner.build_output_stream(
                &cpal_config,
                move |data: &mut [u32], _| {
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
            ),
            cpal::SampleFormat::F32 => self.inner.build_output_stream(
                &cpal_config,
                move |data: &mut [f32], _| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks_mut(channels) {
                        let value = buffer.pop_front().unwrap_or(0.0);
                        for s in frame.iter_mut() {
                            *s = value;
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => return Err(Error::from_reason(format!("Unsupported sample format: {:?}", sample_format))),
        }
        .map_err(|e| Error::from_reason(format!("Failed to build output stream: {}", e)))?;

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
            sample_rate: cpal::SampleRate(config.sample_rate),
            buffer_size: config.buffer_size.into(),
        };

        let channels = config.channels as usize;
        let shared_buffer = buffer.inner.clone();
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let sample_format = config.sample_format.into();

        let stream = match sample_format {
            cpal::SampleFormat::I8 => self.inner.build_input_stream(
                &cpal_config,
                move |data: &[i8], _| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks(channels) {
                        if let Some(sample) = frame.first() {
                            buffer.push_back(cpal::Sample::to_sample::<f32>(*sample));
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U8 => self.inner.build_input_stream(
                &cpal_config,
                move |data: &[u8], _| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks(channels) {
                        if let Some(sample) = frame.first() {
                            buffer.push_back(cpal::Sample::to_sample::<f32>(*sample));
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => self.inner.build_input_stream(
                &cpal_config,
                move |data: &[i16], _| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks(channels) {
                        if let Some(sample) = frame.first() {
                            buffer.push_back(cpal::Sample::to_sample::<f32>(*sample));
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => self.inner.build_input_stream(
                &cpal_config,
                move |data: &[u16], _| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks(channels) {
                        if let Some(sample) = frame.first() {
                            buffer.push_back(cpal::Sample::to_sample::<f32>(*sample));
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I32 => self.inner.build_input_stream(
                &cpal_config,
                move |data: &[i32], _| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks(channels) {
                        if let Some(sample) = frame.first() {
                            buffer.push_back(cpal::Sample::to_sample::<f32>(*sample));
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U32 => self.inner.build_input_stream(
                &cpal_config,
                move |data: &[u32], _| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks(channels) {
                        if let Some(sample) = frame.first() {
                            buffer.push_back(cpal::Sample::to_sample::<f32>(*sample));
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::F32 => self.inner.build_input_stream(
                &cpal_config,
                move |data: &[f32], _| {
                    let mut buffer = shared_buffer.lock().unwrap();
                    for frame in data.chunks(channels) {
                        if let Some(sample) = frame.first() {
                            buffer.push_back(*sample);
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => return Err(Error::from_reason(format!("Unsupported sample format: {:?}", sample_format))),
        }
        .map_err(|e| Error::from_reason(format!("Failed to build input stream: {}", e)))?;

        Ok(AudioStream::new(stream))
    }
}
