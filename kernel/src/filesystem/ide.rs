use core::{panic, sync::atomic::Ordering};

/// IDE Driver Interface responsible for loading and storing data on the disk.
/// You can read more about the driver here: https://wiki.osdev.org/PCI_IDE_Controller
use alloc::sync::Arc;

use crate::{
    apic::{defs::IRQ_IDE, io_apic::enable_irq, local_apic::local_apic_acknowledge},
    devices::pci::{PCIDevice, IS_PCI_MAPPED, PCI_DEVICES},
    scheduler::sleep::wakeup,
    structures::heap_linked_list::HeapLinkedList,
    sync::spin_mutex::SpinMutex,
    x86::helpers::{inb, insd, outb, outsd},
};

static IDE_QUEUE: SpinMutex<HeapLinkedList<Arc<SpinMutex<DiskBlock>>>> =
    SpinMutex::new(HeapLinkedList::new());

const SECTOR_SIZE: usize = 512;
pub const BLOCK_SIZE: usize = 512;
const IDE_BUSY: u8 = 0x80; // Driver is Busy
const IDE_READY: u8 = 0x40; // Driver is Ready
const IDE_FAULT: u8 = 0x20; // Write Fault
const IDE_ERROR: u8 = 0x01; // An Error Occurred

const IDE_READ: u8 = 0x20;
const IDE_WRITE: u8 = 0x30;

const IDE_STATUS_REGISTER: u16 = 0x1F7;
const IDE_COMMAND_REGISTER: u16 = 0x1F7;
const IDE_DRIVE_REGISTER: u16 = 0x1F6;
const IDE_SECTOR_COUNT_REGISTER: u16 = 0x1F2;
const IDE_SECTOR_SELECT_REGISTER: u16 = 0x1F3;
const IDE_CYLINDER_LOW_REGISTER: u16 = 0x1F4;
const IDE_CYLINDER_HIGH_REGISTER: u16 = 0x1F5;
const IDE_DATA_REGISTER: u16 = 0x1F0;
const IDE_CONTROL_REGISTER: u16 = 0x3F6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DiskRequestStatus {
    READY,
    AWAITING,
    FAILED,
}

#[derive(Debug, Clone, Copy)]
pub struct DiskBlock {
    pub dirty: bool,
    pub status: DiskRequestStatus,
    pub device: u32,
    pub block_number: u32,
    pub data: [u8; BLOCK_SIZE],
}

impl DiskBlock {
    pub fn get_address(&self) -> usize {
        self as *const DiskBlock as usize
    }

    pub fn cast_to<T>(&mut self) -> &mut [T] {
        let ptr = self.data.as_mut_ptr();
        let size = core::mem::size_of::<T>();
        let entries = BLOCK_SIZE / size;
        unsafe { core::slice::from_raw_parts_mut(ptr as *mut T, entries) }
    }
}

/// Many IDE operations take time to complete. If the Status Register reports a status of BUSY, the
/// IDE is still processing the last request, and as such we need to wait before requesting the next
/// procedure. This function loops until the IDE responds with a status of READY.
fn wait_ide() -> Result<(), u8> {
    loop {
        let status = inb(IDE_STATUS_REGISTER);

        // Loop until the IDE Driver is ready to receive new requests
        if status & (IDE_BUSY | IDE_READY) == IDE_READY {
            if status & (IDE_FAULT | IDE_ERROR) != 0 {
                return Err(status);
            }

            return Ok(());
        }
    }
}

/// After PCI has been mapped, all devices and functions are pushed to PCI_DEVICES vector. The disk IDE is
/// a PCI device, and as such is visible from the PCI device list. We must acquire the device information
/// in order to establish the running mode (compatibility or native) of the disk device.
unsafe fn find_ide_device() -> Result<PCIDevice, ()> {
    if IS_PCI_MAPPED.load(Ordering::Relaxed) == false {
        return Err(());
    }

    for device in PCI_DEVICES.lock().iter() {
        if device.header.class_code == 0x1 && device.header.subclass == 0x1 {
            return Ok(device.clone());
        }
    }

    Err(())
}

pub fn setup_ide() {
    enable_irq(IRQ_IDE, 0);
    wait_ide().ok();

    // We must first check in which mode is the master IDE controller running
    let device = unsafe { find_ide_device().expect("[ERROR] Could not find IDE Interface") };
    let is_compatibility_mode = device.header.prog_interface & 0x1 == 0;

    if !is_compatibility_mode {
        todo!("[ERROR] IDE Controller in Native PCI Mode not implemented");
    }

    // Check if a secondary disk is present. Send IDENTIFY command to secondary IDE drive
    let mut has_secondary_disk = false;
    outb(0x1F6, 0xF0);
    for _ in 0..1000 {
        if inb(0x1F7) != 0 {
            has_secondary_disk = true;
            break;
        }
    }

    if !has_secondary_disk {
        panic!("[ERROR] No File System Disk");
    }

    // Switch back to primary disk
    outb(0x1F6, 0xE0);
}

pub fn start_ide_request(block: &SpinMutex<DiskBlock>) {
    let block = block.lock();

    let sector_per_block = (BLOCK_SIZE / SECTOR_SIZE) as u32;

    let sector = block.block_number * sector_per_block;
    let device = (block.device & 1) << 4;
    let data = block.data;

    let sector_select = sector & 0xFF;
    let sector_low = (sector >> 8) & 0xFF;
    let sector_high = (sector >> 16_u8) & 0xFF;
    let drive_select = 0xE0 | device | ((sector >> 24) & 0xF);

    wait_ide().ok();

    outb(IDE_CONTROL_REGISTER, 0); // Ask to generate interrupt
    outb(IDE_SECTOR_COUNT_REGISTER, sector_per_block as u8);
    outb(IDE_SECTOR_SELECT_REGISTER, sector_select as u8);
    outb(IDE_CYLINDER_LOW_REGISTER, sector_low as u8);
    outb(IDE_CYLINDER_HIGH_REGISTER, sector_high as u8);
    outb(IDE_DRIVE_REGISTER, drive_select as u8);

    // Switch between writting to disk and reading from disk
    if block.dirty {
        outb(IDE_COMMAND_REGISTER, IDE_WRITE);
        outsd(IDE_DATA_REGISTER, data.as_ptr(), BLOCK_SIZE / 4);
    } else {
        outb(IDE_COMMAND_REGISTER, IDE_READ);
    }
}

/// Once the request has been fulfilled, the IDE performs an interrupt to indicate the data transfer
/// has been completed. For write operations, no additional procedure must be done. For read, the data
/// must be fetched from the IDE buffer.
pub fn interrupt_ide() {
    let mut ide_queue = IDE_QUEUE.lock();
    let ide_block = ide_queue.pop();
    let mut block = ide_block.as_ref().unwrap().lock();

    wait_ide().expect("[ERROR] IDE Failure");

    // Request is a read, must transfer data from IDE buffer
    if block.dirty == false {
        insd(IDE_DATA_REGISTER, block.data.as_ptr(), BLOCK_SIZE / 4);
    }

    block.status = DiskRequestStatus::READY;
    block.dirty = false;

    // Emit wakeup signal to all processes waiting for this block
    wakeup(block.get_address());

    // Request next block to start processing
    if ide_queue.size > 0 {
        let next = ide_queue.peek().unwrap().value.as_ref();
        start_ide_request(next);
    }

    local_apic_acknowledge();
}

/// Request IDE operation, either read or write, as defined by the DiskBlock request.
/// Every request is added to a queue, for which each block is later sent to the IDE to processed.
pub fn request_ide(block: Arc<SpinMutex<DiskBlock>>) {
    let mut ide_queue = IDE_QUEUE.lock();
    ide_queue.push(Arc::clone(&block));

    if ide_queue.size == 1 {
        let next = ide_queue.peek().unwrap().value.as_ref();
        unsafe { IDE_QUEUE.force_unlock() };
        start_ide_request(next);
    }
}
