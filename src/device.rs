use crate::buffer::AudioBuffer;
use crate::config::{BufferSize, StreamConfig, SupportedStreamConfig};
use crate::stream::AudioStream;
use cpal::traits::DeviceTrait;
use napi::bindgen_prelude::*;
use napi_derive::napi;

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
    pub fn description(&self) -> Result<crate::device_description::DeviceDescription> {
        let desc = self
            .inner
            .description()
            .map_err(|e| Error::from_reason(format!("Failed to get device description: {}", e)))?;
        Ok(crate::device_description::DeviceDescription {
            name: desc.name().to_string(),
            direction: match desc.direction() {
                cpal::DeviceDirection::Input => crate::device_description::DeviceDirection::Input,
                cpal::DeviceDirection::Output => crate::device_description::DeviceDirection::Output,
                _ => crate::device_description::DeviceDirection::Output,
            },
            device_type: match desc.device_type() {
                _ => crate::device_description::DeviceType::Other,
            },
            interface_type: match desc.interface_type() {
                _ => crate::device_description::InterfaceType::Other,
            },
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
    pub fn create_beep_stream(&self) -> Result<AudioStream> {
        let config = self.inner.default_output_config().map_err(|e| {
            Error::from_reason(format!("Failed to get default output config: {}", e))
        })?;

        let sample_format = config.sample_format();
        let config_inner: cpal::StreamConfig = config.into();
        let sample_rate = config_inner.sample_rate as f32;
        let channels = config_inner.channels as usize;

        let mut sample_clock = 0f32;
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = match sample_format {
            cpal::SampleFormat::F32 => self.inner.build_output_stream(
                &config_inner,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for frame in data.chunks_mut(channels) {
                        let value = next_value();
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => self.inner.build_output_stream(
                &config_inner,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    for frame in data.chunks_mut(channels) {
                        let value = (next_value() * i16::MAX as f32) as i16;
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::U16 => self.inner.build_output_stream(
                &config_inner,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    for frame in data.chunks_mut(channels) {
                        let value = ((next_value() * 0.5 + 0.5) * u16::MAX as f32) as u16;
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                },
                err_fn,
                None,
            ),
            _ => return Err(Error::from_reason("Unsupported sample format")),
        }
        .map_err(|e| Error::from_reason(format!("Failed to build stream: {}", e)))?;

        Ok(AudioStream::new(stream))
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
