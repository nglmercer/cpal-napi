use napi_derive::napi;

#[napi]
pub enum SampleFormat {
    I16,
    U16,
    F32,
}

#[napi]
#[derive(Debug, Clone, Copy)]
pub enum HostId {
    Alsa,
    Jack,
    Wasapi,
    Asio,
    CoreAudio,
    Emscripten,
    Other,
}

impl From<cpal::HostId> for HostId {
    fn from(id: cpal::HostId) -> Self {
        match id.name().to_lowercase().as_str() {
            "alsa" => HostId::Alsa,
            "jack" => HostId::Jack,
            "wasapi" => HostId::Wasapi,
            "asio" => HostId::Asio,
            "coreaudio" => HostId::CoreAudio,
            "emscripten" => HostId::Emscripten,
            _ => HostId::Other,
        }
    }
}

#[napi]
pub fn get_all_hosts() -> Vec<HostId> {
    vec![
        HostId::Alsa,
        HostId::Jack,
        HostId::Wasapi,
        HostId::Asio,
        HostId::CoreAudio,
        HostId::Emscripten,
    ]
}

