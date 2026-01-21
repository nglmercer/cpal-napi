use cpal::traits::StreamTrait;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct AudioStream {
    stream: Option<cpal::Stream>,
}

impl AudioStream {
    pub fn new(stream: cpal::Stream) -> Self {
        AudioStream {
            stream: Some(stream),
        }
    }
}

#[napi]
impl AudioStream {
    #[napi]
    pub fn play(&self) -> Result<()> {
        if let Some(ref s) = self.stream {
            s.play()
                .map_err(|e| Error::from_reason(format!("Failed to play: {}", e)))
        } else {
            Err(Error::from_reason("Stream is not initialized"))
        }
    }

    #[napi]
    pub fn pause(&self) -> Result<()> {
        if let Some(ref s) = self.stream {
            s.pause()
                .map_err(|e| Error::from_reason(format!("Failed to pause: {}", e)))
        } else {
            Err(Error::from_reason("Stream is not initialized"))
        }
    }
}
