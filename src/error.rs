use napi_derive::napi;

#[napi]
#[derive(Debug, Clone)]
pub enum BuildStreamError {
    DeviceNotAvailable,
    StreamConfigNotSupported,
    InvalidArgument,
    StreamIdOverflow,
    BackendSpecific,
}

#[napi]
#[derive(Debug, Clone)]
pub enum DefaultStreamConfigError {
    DeviceNotAvailable,
    StreamConfigNotSupported,
    BackendSpecific,
}

#[napi]
#[derive(Debug, Clone)]
pub enum DeviceIdError {
    DeviceNotAvailable,
    BackendSpecific,
}

#[napi]
#[derive(Debug, Clone)]
pub enum DeviceNameError {
    DeviceNotAvailable,
    BackendSpecific,
}

#[napi]
#[derive(Debug, Clone)]
pub enum DevicesError {
    BackendSpecific,
}

#[napi]
#[derive(Debug, Clone)]
pub enum PauseStreamError {
    DeviceNotAvailable,
    BackendSpecific,
}

#[napi]
#[derive(Debug, Clone)]
pub enum PlayStreamError {
    DeviceNotAvailable,
    BackendSpecific,
}

#[napi]
#[derive(Debug, Clone)]
pub enum StreamError {
    DeviceNotAvailable,
    BackendSpecific,
}

#[napi]
#[derive(Debug, Clone)]
pub enum SupportedStreamConfigsError {
    DeviceNotAvailable,
    InvalidArgument,
    BackendSpecific,
}

#[napi(object)]
pub struct BackendSpecificError {
    pub description: String,
}

#[napi(object)]
pub struct HostUnavailable;
