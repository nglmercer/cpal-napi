use cpal::traits::StreamTrait;
use napi::bindgen_prelude::*;

use napi_derive::napi;

#[napi(object)]
pub struct StreamInstant {
    pub seconds: i64,
    pub nanos: u32,
}

#[napi(object)]
pub struct InputStreamTimestamp {
    pub callback: StreamInstant,
    pub capture: StreamInstant,
}

#[napi(object)]
pub struct OutputStreamTimestamp {
    pub callback: StreamInstant,
    pub playback: StreamInstant,
}

#[napi(object)]
pub struct InputCallbackInfo {
    pub timestamp: InputStreamTimestamp,
}

#[napi(object)]
pub struct OutputCallbackInfo {
    pub timestamp: OutputStreamTimestamp,
}

#[napi]
pub struct AudioStream {
    pub(crate) stream: Option<cpal::Stream>,
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

    #[napi]
    pub fn close(&mut self) {
        if let Some(s) = self.stream.take() {
            let _ = s.pause();
            drop(s);
        }
    }
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        // Ensure stream is stopped and resources are freed when the object is GC'd
        if let Some(s) = self.stream.take() {
            let _ = s.pause();
            drop(s);
        }
    }
}
