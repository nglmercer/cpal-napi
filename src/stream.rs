use cpal::traits::StreamTrait;
use napi::bindgen_prelude::*;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;

use napi_derive::napi;

// Global tracking of active streams for graceful shutdown
static STREAM_COUNTER: AtomicU64 = AtomicU64::new(1);
static SHUTDOWN_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
    static ref ACTIVE_STREAMS: Mutex<HashMap<u64, ()>> = Mutex::new(HashMap::new());
}

/// Call this before process exit to gracefully clean up all audio streams.
/// This prevents double-free errors that can occur when ALSA cleanup races
/// with the Node.js/Bun runtime shutdown.
#[napi]
pub fn prepare_shutdown() {
    SHUTDOWN_IN_PROGRESS.store(true, Ordering::SeqCst);
    
    // Give a small amount of time for any in-flight audio callbacks to complete
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    // Clear the active streams tracking
    if let Ok(mut streams) = ACTIVE_STREAMS.lock() {
        streams.clear();
    }
    
    // Additional sleep to let ALSA threads settle
    std::thread::sleep(std::time::Duration::from_millis(100));
}

/// Check if shutdown is in progress
pub fn is_shutdown_in_progress() -> bool {
    SHUTDOWN_IN_PROGRESS.load(Ordering::SeqCst)
}

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
    stream_id: u64,
}

impl AudioStream {
    pub fn new(stream: cpal::Stream) -> Self {
        let id = STREAM_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // Track this stream
        if let Ok(mut streams) = ACTIVE_STREAMS.lock() {
            streams.insert(id, ());
        }
        
        AudioStream {
            stream: Some(stream),
            stream_id: id,
        }
    }
    
    fn cleanup(&mut self) {
        if let Some(s) = self.stream.take() {
            // Untrack this stream
            if let Ok(mut streams) = ACTIVE_STREAMS.lock() {
                streams.remove(&self.stream_id);
            }
            
            // Pause first, then let it drop naturally
            let _ = s.pause();
            
            // Small delay to let ALSA finish any pending callbacks
            if !is_shutdown_in_progress() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            
            drop(s);
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
        self.cleanup();
    }
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        self.cleanup();
    }
}
