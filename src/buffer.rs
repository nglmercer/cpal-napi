use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[napi]
pub struct AudioBuffer {
    pub(crate) inner: Arc<Mutex<VecDeque<f32>>>,
}

impl Default for AudioBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl AudioBuffer {
    #[napi(constructor)]
    pub fn new() -> Self {
        AudioBuffer {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(44100))),
        }
    }

    #[napi]
    pub fn push(&self, data: Float32Array) {
        let mut buffer = self.inner.lock().unwrap();
        for i in 0..data.len() {
            buffer.push_back(data[i]);
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
}
