use alloc::vec::{self, Vec};

use crate::{
    println,
    sync::spin_mutex::SpinMutex,
    x86::helpers::{inw, outw},
};

const PCI_CONFIG_REGISTER: u16 = 0xCF8;
const PCI_DATA_REGISTER: u16 = 0xCFC;

pub static PCI_DEVICES: SpinMutex<Vec<PCIDevice>> = SpinMutex::new(Vec::new());
pub static mut IS_PCI_MAPPED: bool = false;

#[derive(Debug, Clone, Copy)]
pub struct PCIDevice {
    bus: u8,
    device: u8,
    function: u8,
    pub header: PCIHeader,
    body_0: Option<TypeZeroDeviceTable>,
}

#[derive(Debug, Clone, Copy)]
pub struct PCIHeader {
    // Register 0
    pub vendor_id: u16,
    pub device_id: u16,

    // Register 1
    pub status: u16,
    pub command: u16,

    // Register 2
    pub class_code: u8,
    pub subclass: u8,
    pub prog_interface: u8,

    // Register 3
    pub header_type: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct TypeZeroDeviceTable {
    // Base Address Registers
    bar_0: u32,
    bar_1: u32,
    bar_2: u32,
    bar_3: u32,
    bar_4: u32,
}

/// Reads 32 bits from the provided register in the specified bus and device.
fn pci_read_dword(bus: u8, device: u8, function: u8, register: u8) -> u32 {
    let bus = bus as u32;
    let slot = device as u32;
    let function = function as u32;
    let offset = (register * 4) as u32;

    // Write the address
    let address = 0x80000000 | bus << 16 | slot << 11 | function << 8 | (offset & 0xFC);
    outw(PCI_CONFIG_REGISTER, address);

    // Read the data in the address
    return inw(PCI_DATA_REGISTER);
}

fn get_type_zero_table(bus: u8, device: u8, function: u8) -> TypeZeroDeviceTable {
    TypeZeroDeviceTable {
        bar_0: pci_read_dword(bus, device, function, 4),
        bar_1: pci_read_dword(bus, device, function, 5),
        bar_2: pci_read_dword(bus, device, function, 6),
        bar_3: pci_read_dword(bus, device, function, 7),
        bar_4: pci_read_dword(bus, device, function, 8),
    }
}

fn get_header(bus: u8, device: u8, function: u8) -> Result<PCIHeader, ()> {
    let reg0 = pci_read_dword(bus, device, function, 0);

    // Vendor ID is undefined, no device here
    if reg0 & 0xFFFF == 0xFFFF {
        return Err(());
    }

    let reg1 = pci_read_dword(bus, device, function, 1);
    let reg2 = pci_read_dword(bus, device, function, 2);
    let reg3 = pci_read_dword(bus, device, function, 3);

    let header = PCIHeader {
        vendor_id: reg0 as u16 & 0xFFFF,
        device_id: (reg0 >> 16) as u16 & 0xFFFF,
        command: reg1 as u16 & 0xFFFF,
        status: (reg1 >> 16) as u16 & 0xFFFF,
        class_code: (reg2 >> 24) as u8 & 0xFF,
        subclass: (reg2 >> 16) as u8 & 0xFF,
        prog_interface: (reg2 >> 8) as u8 & 0xFF,
        header_type: (reg3 >> 16) as u8 & 0xFF,
    };

    Ok(header)
}

fn list_pci_device(bus: u8, device: u8, function: u8) {
    let Ok(header) = get_header(bus, device, function) else {
        return;
    };

    let is_multi_function = header.header_type & 0x80 != 0;
    if is_multi_function {
        for _function in 1..8 {
            list_pci_device(bus, device, _function);
        }
    }

    // Get Base Address Registers if this is a common device
    let body_0 = if header.header_type == 0 {
        Some(get_type_zero_table(bus, device, function))
    } else {
        None
    };

    let pci_device = PCIDevice {
        header,
        bus,
        device,
        function,
        body_0,
    };

    unsafe { PCI_DEVICES.lock().push(pci_device) };
}

pub fn map_pci_buses() {
    for bus_index in 0..256 {
        for device_index in 0..32 {
            list_pci_device(bus_index as u8, device_index as u8, 0);
        }
    }

    println!("[KERNEL] PCI Devices Mapped");

    unsafe {
        IS_PCI_MAPPED = true;
    }
}
