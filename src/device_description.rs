use napi_derive::napi;

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceDirection {
    Input,
    Output,
}

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Internal,
    Usb,
    Bluetooth,
    Network,
    Firewire,
    Virtual,
    Other,
}

#[napi]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceType {
    Alsa,
    Jack,
    Wasapi,
    Asio,
    CoreAudio,
    Emscripten,
    Other,
}

#[napi(object)]
pub struct DeviceDescription {
    pub name: String,
    pub direction: DeviceDirection,
    pub device_type: DeviceType,
    pub interface_type: InterfaceType,
}

#[napi]
pub struct DeviceDescriptionBuilder {
    name: Option<String>,
    direction: Option<DeviceDirection>,
    device_type: Option<DeviceType>,
    interface_type: Option<InterfaceType>,
}

impl Default for DeviceDescriptionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl DeviceDescriptionBuilder {
    #[napi(constructor)]
    pub fn new() -> DeviceDescriptionBuilder {
        DeviceDescriptionBuilder {
            name: None,
            direction: None,
            device_type: None,
            interface_type: None,
        }
    }

    #[napi]
    pub fn name(&mut self, name: String) {
        self.name = Some(name);
    }

    #[napi]
    pub fn direction(&mut self, direction: DeviceDirection) {
        self.direction = Some(direction);
    }

    #[napi]
    pub fn device_type(&mut self, device_type: DeviceType) {
        self.device_type = Some(device_type);
    }

    #[napi]
    pub fn interface_type(&mut self, interface_type: InterfaceType) {
        self.interface_type = Some(interface_type);
    }

    #[napi]
    pub fn build(&self) -> DeviceDescription {
        DeviceDescription {
            name: self.name.clone().unwrap_or_default(),
            direction: self.direction.unwrap_or(DeviceDirection::Output),
            device_type: self.device_type.unwrap_or(DeviceType::Other),
            interface_type: self.interface_type.unwrap_or(InterfaceType::Other),
        }
    }
}
