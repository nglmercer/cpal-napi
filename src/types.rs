use napi_derive::napi;

#[napi]
pub enum SampleFormat {
    I16,
    U16,
    F32,
}

#[napi]
#[derive(Debug)]
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
pub enum BufferSize {
    Default,
    Fixed,
}

#[napi(object)]
pub struct SupportedBufferSize {
    pub min: u32,
    pub max: u32,
}

#[napi]
pub enum DeviceDirection {
    Input,
    Output,
}

#[napi]
pub enum DeviceType {
    BuiltIn,
    Usb,
    Bluetooth,
    Hdmi,
    Other,
}

#[napi(object)]
pub struct StreamInstant {
    pub secs: i64,
    pub nanos: u32,
}

#[napi(object)]
pub struct OutputStreamTimestamp {
    pub callback: StreamInstant,
    pub playback: StreamInstant,
}

#[napi(object)]
pub struct InputStreamTimestamp {
    pub callback: StreamInstant,
    pub capture: StreamInstant,
}

#[napi(object)]
pub struct I24 {
    pub value: i32,
}

#[napi(object)]
pub struct U24 {
    pub value: u32,
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
