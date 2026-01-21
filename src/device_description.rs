use crate::types::{DeviceDirection, DeviceType};
use napi_derive::napi;

#[napi(object)]
pub struct DeviceDescription {
    pub name: String,
    pub direction: DeviceDirection,
    pub device_type: DeviceType,
}
