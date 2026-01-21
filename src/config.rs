use crate::types::SampleFormat;
use napi_derive::napi;

#[napi(object)]
#[derive(Clone, Copy)]
pub struct StreamConfig {
    pub channels: u16,
    pub sample_rate: u32,
    pub buffer_size: u32, // 0 for default
}

#[napi(object)]
pub struct SupportedStreamConfig {
    pub channels: u16,
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub sample_format: SampleFormat,
}

impl From<cpal::SupportedStreamConfigRange> for SupportedStreamConfig {
    fn from(c: cpal::SupportedStreamConfigRange) -> Self {
        SupportedStreamConfig {
            channels: c.channels(),
            min_sample_rate: c.min_sample_rate().0,
            max_sample_rate: c.max_sample_rate().0,
            sample_format: match c.sample_format() {
                cpal::SampleFormat::I16 => SampleFormat::I16,
                cpal::SampleFormat::U16 => SampleFormat::U16,
                cpal::SampleFormat::F32 => SampleFormat::F32,
                _ => SampleFormat::F32,
            },
        }
    }
}
