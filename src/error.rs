use napi_derive::napi;

#[napi]
pub enum BuildStreamError {
    DeviceNotAvailable,
    StreamConfigNotSupported,
    InvalidArgument,
    BackendSpecific,
}

#[napi]
pub enum PlayStreamError {
    DeviceNotAvailable,
    BackendSpecific,
}

#[napi]
pub enum PauseStreamError {
    DeviceNotAvailable,
    BackendSpecific,
}

#[napi]
pub enum DefaultStreamConfigError {
    DeviceNotAvailable,
    StreamTypeNotSupported,
    BackendSpecific,
}

#[napi]
pub enum DevicesError {
    BackendSpecific,
}

#[napi]
pub enum DeviceNameError {
    DeviceNotAvailable,
    BackendSpecific,
}

#[napi]
pub enum StreamError {
    BackendSpecific,
}

#[napi]
pub enum SupportedStreamConfigsError {
    DeviceNotAvailable,
    BackendSpecific,
}
