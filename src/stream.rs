use cpal::traits::StreamTrait;
use napi::bindgen_prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use napi_derive::napi;

// Global tracking of active streams for graceful shutdown
static STREAM_COUNTER: AtomicU64 = AtomicU64::new(1);
static SHUTDOWN_IN_PROGRESS: AtomicBool = AtomicBool::new(false);
static ACTIVE_STREAM_COUNT: AtomicUsize = AtomicUsize::new(0);

lazy_static::lazy_static! {
    static ref ACTIVE_STREAMS: Mutex<HashMap<u64, ()>> = Mutex::new(HashMap::new());
}

/// Call this before process exit to gracefully clean up all audio streams.
/// This prevents double-free errors that can occur when ALSA cleanup races
/// with the Node.js/Bun runtime shutdown.
#[napi]
pub fn prepare_shutdown() {
    SHUTDOWN_IN_PROGRESS.store(true, Ordering::SeqCst);

    // Memory barrier to ensure all threads see the shutdown flag
    std::sync::atomic::fence(Ordering::SeqCst);

    // Wait for active callbacks to complete (up to 500ms)
    let start = std::time::Instant::now();
    while ACTIVE_STREAM_COUNT.load(Ordering::SeqCst) > 0 {
        if start.elapsed() > std::time::Duration::from_millis(500) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Additional sleep to let ALSA threads fully settle
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Clear the active streams tracking
    if let Ok(mut streams) = ACTIVE_STREAMS.lock() {
        streams.clear();
    }
}

/// Check if shutdown is in progress
pub fn is_shutdown_in_progress() -> bool {
    SHUTDOWN_IN_PROGRESS.load(Ordering::SeqCst)
}

/// Get the number of active streams
#[napi]
pub fn get_active_stream_count() -> u32 {
    ACTIVE_STREAM_COUNT.load(Ordering::SeqCst) as u32
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
    is_playing: AtomicBool,
}

impl AudioStream {
    pub fn new(stream: cpal::Stream) -> Self {
        let id = STREAM_COUNTER.fetch_add(1, Ordering::SeqCst);

        // Track this stream
        if let Ok(mut streams) = ACTIVE_STREAMS.lock() {
            streams.insert(id, ());
        }
        ACTIVE_STREAM_COUNT.fetch_add(1, Ordering::SeqCst);

        AudioStream {
            stream: Some(stream),
            stream_id: id,
            is_playing: AtomicBool::new(false),
        }
    }

    fn cleanup(&mut self) {
        if let Some(s) = self.stream.take() {
            // Untrack this stream
            if let Ok(mut streams) = ACTIVE_STREAMS.lock() {
                streams.remove(&self.stream_id);
            }

            // Mark as not playing
            self.is_playing.store(false, Ordering::SeqCst);

            // Pause first to stop callbacks
            let _ = s.pause();

            // If shutdown is in progress, we need to be extra careful
            // Use mem::forget to prevent ALSA from trying to free resources
            // that might be racing with the runtime shutdown
            if is_shutdown_in_progress() {
                // During shutdown, we "leak" the stream to prevent double-free
                // The OS will clean up memory when the process exits anyway
                std::mem::forget(s);
            } else {
                // Normal cleanup - give ALSA time to finish callbacks
                std::thread::sleep(std::time::Duration::from_millis(20));
                drop(s);
            }

            // Decrement active count after cleanup
            ACTIVE_STREAM_COUNT.fetch_sub(1, Ordering::SeqCst);
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
