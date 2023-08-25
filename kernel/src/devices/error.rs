#[derive(Copy, Clone, Debug)]
pub enum SerialError {
    PortUnavailable,
}

#[derive(Copy, Clone, Debug)]
pub enum PCIError {
    DeviceNotFound,
}
