use napi_derive::napi;

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
}

impl From<cpal::SampleFormat> for SampleFormat {
    fn from(format: cpal::SampleFormat) -> Self {
        match format {
            cpal::SampleFormat::I8 => SampleFormat::I8,
            cpal::SampleFormat::U8 => SampleFormat::U8,
            cpal::SampleFormat::I16 => SampleFormat::I16,
            cpal::SampleFormat::U16 => SampleFormat::U16,
            cpal::SampleFormat::I32 => SampleFormat::I32,
            cpal::SampleFormat::U32 => SampleFormat::U32,
            cpal::SampleFormat::I64 => SampleFormat::I64,
            cpal::SampleFormat::U64 => SampleFormat::U64,
            cpal::SampleFormat::F32 => SampleFormat::F32,
            cpal::SampleFormat::F64 => SampleFormat::F64,
            _ => SampleFormat::F32,
        }
    }
}

impl From<SampleFormat> for cpal::SampleFormat {
    fn from(format: SampleFormat) -> Self {
        match format {
            SampleFormat::I8 => cpal::SampleFormat::I8,
            SampleFormat::U8 => cpal::SampleFormat::U8,
            SampleFormat::I16 => cpal::SampleFormat::I16,
            SampleFormat::U16 => cpal::SampleFormat::U16,
            SampleFormat::I32 => cpal::SampleFormat::I32,
            SampleFormat::U32 => cpal::SampleFormat::U32,
            SampleFormat::I64 => cpal::SampleFormat::I64,
            SampleFormat::U64 => cpal::SampleFormat::U64,
            SampleFormat::F32 => cpal::SampleFormat::F32,
            SampleFormat::F64 => cpal::SampleFormat::F64,
        }
    }
}

pub type ChannelCount = u16;
pub type FrameCount = u32;
pub type SampleRate = u32;

pub type DevicesFiltered = Vec<crate::device::AudioDevice>;
pub type InputDevices = Vec<crate::device::AudioDevice>;
pub type OutputDevices = Vec<crate::device::AudioDevice>;

#[napi]
pub struct I24 {
    pub(crate) inner: i32,
}

#[napi]
impl I24 {
    #[napi(constructor)]
    pub fn new(value: i32) -> Self {
        Self {
            inner: value & 0xFFFFFF,
        }
    }

    #[napi]
    pub fn to_i32(&self) -> i32 {
        self.inner
    }
}

#[napi]
pub struct U24 {
    pub(crate) inner: u32,
}

#[napi]
impl U24 {
    #[napi(constructor)]
    pub fn new(value: u32) -> Self {
        Self {
            inner: value & 0xFFFFFF,
        }
    }

    #[napi]
    pub fn to_u32(&self) -> u32 {
        self.inner
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i24() {
        let val = I24::new(0x12345678);
        assert_eq!(val.to_i32(), 0x345678);
    }

    #[test]
    fn test_u24() {
        let val = U24::new(0x12345678);
        assert_eq!(val.to_u32(), 0x345678);
    }
}
