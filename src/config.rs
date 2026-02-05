use crate::types::SampleFormat;
use napi_derive::napi;

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSize {
    Default,
    Fixed(u32),
}

impl From<cpal::BufferSize> for BufferSize {
    fn from(size: cpal::BufferSize) -> Self {
        match size {
            cpal::BufferSize::Default => BufferSize::Default,
            cpal::BufferSize::Fixed(s) => BufferSize::Fixed(s),
        }
    }
}

impl From<BufferSize> for cpal::BufferSize {
    fn from(size: BufferSize) -> Self {
        match size {
            BufferSize::Default => cpal::BufferSize::Default,
            BufferSize::Fixed(s) => cpal::BufferSize::Fixed(s),
        }
    }
}

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelMixMode {
    Auto,     // Dynamically pick the active channel (default)
    Left,     // Force use of Left channel (1st)
    Right,    // Force use of Right channel (2nd)
    Balanced, // Simple average (L+R)/2
}

#[napi(object)]
#[derive(Clone, Copy)]
pub struct StreamConfig {
    pub channels: u16,
    pub sample_rate: u32,
    pub buffer_size: BufferSize,
    pub sample_format: SampleFormat,
    pub mix_mode: Option<ChannelMixMode>,
    /// Noise gate threshold (0.0 to 1.0). Samples below this level are silenced.
    /// Default is None (no noise gate). Recommended range: 0.01-0.05 for hiss reduction.
    pub noise_gate_threshold: Option<f64>,
}

impl From<cpal::StreamConfig> for StreamConfig {
    fn from(c: cpal::StreamConfig) -> Self {
        StreamConfig {
            channels: c.channels,
            sample_rate: c.sample_rate,
            buffer_size: c.buffer_size.into(),
            sample_format: SampleFormat::F32, // Default to F32 when converting from cpal::StreamConfig which doesn't have format
            mix_mode: Some(ChannelMixMode::Auto),
            noise_gate_threshold: None,
        }
    }
}

impl From<cpal::SupportedStreamConfig> for StreamConfig {
    fn from(c: cpal::SupportedStreamConfig) -> Self {
        let sample_format = c.sample_format().into();
        let config = c.config();
        StreamConfig {
            channels: config.channels,
            sample_rate: config.sample_rate,
            buffer_size: config.buffer_size.into(),
            sample_format,
            mix_mode: Some(ChannelMixMode::Auto),
            noise_gate_threshold: None,
        }
    }
}

impl From<StreamConfig> for cpal::StreamConfig {
    fn from(c: StreamConfig) -> Self {
        cpal::StreamConfig {
            channels: c.channels,
            sample_rate: c.sample_rate,
            buffer_size: c.buffer_size.into(),
        }
    }
}

#[napi(object)]
pub struct SupportedStreamConfig {
    pub channels: u16,
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub buffer_size: SupportedBufferSize,
    pub sample_format: SampleFormat,
}

impl From<cpal::SupportedStreamConfigRange> for SupportedStreamConfig {
    fn from(c: cpal::SupportedStreamConfigRange) -> Self {
        SupportedStreamConfig {
            channels: c.channels(),
            min_sample_rate: c.min_sample_rate(),
            max_sample_rate: c.max_sample_rate(),
            buffer_size: (*c.buffer_size()).into(),
            sample_format: c.sample_format().into(),
        }
    }
}

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedBufferSize {
    Unknown,
    Range(u32, u32),
}

impl From<cpal::SupportedBufferSize> for SupportedBufferSize {
    fn from(size: cpal::SupportedBufferSize) -> Self {
        match size {
            cpal::SupportedBufferSize::Unknown => SupportedBufferSize::Unknown,
            cpal::SupportedBufferSize::Range { min, max } => SupportedBufferSize::Range(min, max),
        }
    }
}

#[napi(object)]
pub struct SupportedStreamConfigRange {
    pub channels: u16,
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub buffer_size: SupportedBufferSize,
    pub sample_format: SampleFormat,
}

impl From<cpal::SupportedStreamConfigRange> for SupportedStreamConfigRange {
    fn from(c: cpal::SupportedStreamConfigRange) -> Self {
        SupportedStreamConfigRange {
            channels: c.channels(),
            min_sample_rate: c.min_sample_rate(),
            max_sample_rate: c.max_sample_rate(),
            buffer_size: (*c.buffer_size()).into(),
            sample_format: c.sample_format().into(),
        }
    }
}
