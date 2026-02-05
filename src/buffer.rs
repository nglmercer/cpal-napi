use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

type BufferInner = Arc<Mutex<VecDeque<f32>>>;
type LinksVec = Arc<Mutex<Vec<BufferInner>>>;

#[napi]
pub struct AudioBuffer {
    pub(crate) inner: BufferInner,
    pub(crate) links: LinksVec,
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioBuffer {
    fn drop(&mut self) {
        if let Ok(mut links) = self.links.lock() {
            links.clear();
        }
    }
}

#[napi]
impl AudioBuffer {
    #[napi(constructor)]
    pub fn new() -> Self {
        AudioBuffer {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(44100))),
            links: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[napi]
    pub fn clone_link(&self) -> Self {
        let new_inner = Arc::new(Mutex::new(VecDeque::with_capacity(44100)));
        let mut links = self.links.lock().unwrap();
        links.push(new_inner.clone());
        AudioBuffer {
            inner: new_inner,
            links: Arc::new(Mutex::new(Vec::new())), // The clone doesn't inherit parent's links
        }
    }

    #[napi]
    pub fn push(&self, data: Float32Array) {
        let data_ref = data.as_ref();

        // Push to main buffer
        {
            let mut buffer = self.inner.lock().unwrap();
            buffer.extend(data_ref);
        }

        // Broadcast to links
        {
            let links = self.links.lock().unwrap();
            for link in links.iter() {
                let mut buffer = link.lock().unwrap();
                buffer.extend(data_ref);
            }
        }
    }

    #[napi]
    pub fn beep(&self, frequency: f64, duration_ms: f64, sample_rate: f64) {
        let num_samples = (sample_rate * (duration_ms / 1000.0)) as usize;
        let mut samples = Vec::with_capacity(num_samples);
        let pi2 = 2.0 * std::f64::consts::PI;
        for i in 0..num_samples {
            let t = i as f64 / sample_rate;
            let sample = (t * frequency * pi2).sin() as f32;
            samples.push(sample);
        }

        // Push to main
        {
            let mut buffer = self.inner.lock().unwrap();
            buffer.extend(&samples);
        }

        // Push to links
        {
            let links = self.links.lock().unwrap();
            for link in links.iter() {
                let mut buffer = link.lock().unwrap();
                buffer.extend(&samples);
            }
        }
    }

    #[napi]
    pub fn clear(&self) {
        let mut buffer = self.inner.lock().unwrap();
        buffer.clear();
    }

    #[napi]
    pub fn length(&self) -> u32 {
        let buffer = self.inner.lock().unwrap();
        buffer.len() as u32
    }

    #[napi]
    pub fn get_data(&self) -> Float32Array {
        let buffer = self.inner.lock().unwrap();
        let vec: Vec<f32> = buffer.iter().cloned().collect();
        Float32Array::from(vec)
    }

    /// Check if the buffer contains only silence (all samples below threshold)
    #[napi]
    pub fn is_silent(&self, threshold: Option<f64>) -> bool {
        let threshold = threshold.unwrap_or(0.001) as f32;
        let buffer = self.inner.lock().unwrap();
        buffer.iter().all(|&s| s.abs() < threshold)
    }

    /// Get the peak level (maximum absolute amplitude) of the buffer
    #[napi]
    pub fn get_peak_level(&self) -> f32 {
        let buffer = self.inner.lock().unwrap();
        buffer.iter().map(|s| s.abs()).fold(0.0_f32, f32::max)
    }

    /// Clear the buffer if all samples are below the silence threshold
    /// Returns true if the buffer was cleared, false otherwise
    #[napi]
    pub fn clear_if_silent(&self, threshold: Option<f64>) -> bool {
        let threshold = threshold.unwrap_or(0.001) as f32;
        let mut buffer = self.inner.lock().unwrap();
        let is_silent = buffer.iter().all(|&s| s.abs() < threshold);
        if is_silent {
            buffer.clear();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_buffer() {
        let buffer = AudioBuffer::new();
        assert_eq!(buffer.length(), 0);

        // We can't easily create Float32Array in Rust tests without napi context
        // but we can test the inner logic if we want, or just verify it doesn't crash
        buffer.clear();
        assert_eq!(buffer.length(), 0);
    }
}
