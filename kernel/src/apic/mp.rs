use core::{
    mem::size_of,
    sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
};

use lazy_static::lazy_static;

use crate::{
    memory::defs::{GlobalDescriptorTable, TaskStateSegment, KERNEL_BASE, MEM_BDA},
    println,
    scheduler::defs::process::Context,
    sync::spin_mutex::SpinMutex,
    x86::helpers::{inb, outb},
    P2V,
};

use super::{io_apic::IOApic, local_apic::get_local_apic_id};

pub const MAX_NUM_CPUS: usize = 8;
pub const MP_PROCESS: u8 = 0x0;
pub const MP_BUS: u8 = 0x01;
pub const MP_IO_APIC: u8 = 0x2;
pub const MP_IO_INTERRUPT: u8 = 0x3;
pub const MP_LOCAL_INTERRUPT: u8 = 0x4;

pub static LOCAL_APIC: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    pub static ref CPUS: [Option<CPU>; MAX_NUM_CPUS] = setup_mp();
}

pub static IS_CPU_MAPPED: AtomicBool = AtomicBool::new(false);
pub static mut IO_APIC: SpinMutex<Option<*mut IOApic>> = SpinMutex::new(None);

#[derive(Debug)]
pub struct CPU {
    pub apic_id: u8,
    pub context: SpinMutex<Context>,
    pub taskstate: SpinMutex<Option<TaskStateSegment>>,
    pub gdt: SpinMutex<GlobalDescriptorTable>,
    pub number_cli: AtomicU32, // Number of CLI (Clear Interrupt) issued
    pub enable_interrupt: AtomicBool, // State of interrupts before pushcli
}

unsafe impl Sync for CPU {}
unsafe impl Send for CPU {}

#[repr(C)]
#[derive(Debug)]
struct MPProcess {
    _type: u8,
    apic_id: u8,
    version: u8,
    flags: u8,
    signature: [u8; 4],
    feature: usize,
    reserved: [u8; 8],
}

#[repr(C)]
#[derive(Debug)]
pub struct MPIOApic {
    _type: u8,
    apic_number: u8,
    version: u8,
    flags: u8,
    address: *const usize,
}

#[repr(C)]
#[derive(Debug)]
struct MPFPStructure {
    signature: [u8; 4], // _MP_
    address: usize,
    length: u8,
    revision: u8,
    checksum: u8,
    _type: u8,
    imcp: u8,
    reseved: [u8; 3],
}

#[repr(C)]
#[derive(Debug)]
struct MPConfigTable {
    signature: [u8; 4], // PCMP
    length: u16,
    version: u8,
    checksum: u8,
    product: [u8; 20],
    oem_table: usize,
    oem_length: u16,
    entry: u16,
    local_apic_address: usize, // Access to this address allows for access to the APIC
    extended_length: u16,
    extended_checksum: u8,
    reserved: u8,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            apic_id: 0,
            context: SpinMutex::new(Default::default()),
            gdt: SpinMutex::new(GlobalDescriptorTable::new()),
            taskstate: SpinMutex::new(None),
            enable_interrupt: AtomicBool::new(false),
            number_cli: AtomicU32::new(0),
        }
    }

    pub fn get_cli(&self) -> u32 {
        self.number_cli.load(Ordering::Relaxed)
    }

    pub fn set_cli(&self, value: u32) {
        self.number_cli.store(value, Ordering::Relaxed);
    }

    pub fn get_interrupt_state(&self) -> bool {
        self.enable_interrupt.load(Ordering::Relaxed)
    }

    pub fn set_interrupt_state(&self, value: bool) {
        self.enable_interrupt.store(value, Ordering::Relaxed);
    }
}

pub fn get_my_cpu<'a>() -> &'a CPU {
    let apic_id = get_local_apic_id() as u8;

    for cpu in CPUS.iter() {
        if cpu.is_some() && cpu.as_ref().unwrap().apic_id == apic_id {
            return cpu.as_ref().unwrap();
        }
    }

    panic!("[FATAL] CPU has not been found");
}

pub unsafe fn check_sum(address: *const u8, length: usize) -> u8 {
    let mut sum: usize = 0;
    for i in 0..length {
        sum += *address.offset(i as isize) as usize;
    }
    return sum as u8;
}

pub fn setup_cpus() -> *const Option<CPU> {
    CPUS.as_ptr()
}

pub fn setup_mp() -> [Option<CPU>; MAX_NUM_CPUS] {
    let mp_table = unsafe { find_mp_table().as_ref().unwrap() };
    let mp_conf = unsafe { find_mp_config(mp_table).unwrap() };

    // Store local apic address for the CPU
    let local_apic_address = unsafe { (*mp_conf).local_apic_address };
    LOCAL_APIC.store(local_apic_address, Ordering::Relaxed);

    if mp_table.imcp > 0 {
        // Interrupt Mode Configuration Register
        outb(0x22, 0x70); // Select IMCR
        outb(0x23, inb(0x23) | 1); // Pass-through NMI interrupts
    }

    println!("[KERNEL] Multiprocessing Tables Fetched");

    // Extract APIC fields from config table
    return unsafe { parse_config_table(mp_conf) };
}

unsafe fn parse_config_table(config_table: *const MPConfigTable) -> [Option<CPU>; MAX_NUM_CPUS] {
    let mut start = config_table.offset(1) as *const u8;
    let end = (config_table as *const u8).offset((*config_table).length as isize);
    let mut number_cpus = 0;

    const VALUE: Option<CPU> = None;
    let mut cpus: [Option<CPU>; MAX_NUM_CPUS] = [VALUE; MAX_NUM_CPUS];

    while start < end {
        match *start {
            // Defines a CPU. We must index that CPU for later use
            MP_PROCESS => {
                if number_cpus < MAX_NUM_CPUS {
                    let process = start as *const MPProcess;
                    cpus[number_cpus] = Some(CPU::new());
                    cpus[number_cpus].as_mut().unwrap().apic_id = (*process).apic_id;
                    number_cpus += 1;
                }

                start = start.offset(size_of::<MPProcess>() as isize);
                continue;
            }

            // Defines the global IO APIC. Should be stored for later usage
            MP_IO_APIC => {
                let io_apic = start as *const MPIOApic;
                *IO_APIC.lock() = Some((*io_apic).address as *mut IOApic);
                start = start.offset(size_of::<MPIOApic>() as isize);
                continue;
            }

            MP_BUS | MP_IO_INTERRUPT | MP_LOCAL_INTERRUPT => {
                start = start.offset(8);
                continue;
            }

            // Undefined entry
            _ => {}
        }
    }

    return cpus;
}

unsafe fn find_mp_config(mp_table: &MPFPStructure) -> Option<*const MPConfigTable> {
    if mp_table.address == 0 {
        panic!("[FATAL] MP invalid address");
    }

    let config = P2V!(mp_table.address) as *const MPConfigTable;

    // Check for Config Table signature (PCMP)
    if (*config).signature.as_slice() != b"PCMP" {
        return None;
    }

    // Check if version is the one expected
    if (*config).version != 1 && (*config).version != 4 {
        return None;
    }

    // Perform checksum on the structure
    if check_sum(config as *const u8, (*config).length as usize) != 0 {
        return None;
    }

    return Some(config);
}

/// To get started with the Multiprocessing Specifications, we need to find the MP Floating
/// Pointer Structure. It can be in one of the following locations:
/// - First KB of the EBDA (Extended Bios Data Area)
/// - Last KB of the System Base Memory
/// - In the BIOS ROM between 0xF0000 and 0xFFFFF
unsafe fn find_mp_table() -> *const MPFPStructure {
    let bda = P2V!(MEM_BDA) as *const u8;

    // Get first KB of the EBDA from the BDA
    let address_upper_half = *bda.offset(0xF) as usize;
    let address_lower_half = *bda.offset(0xE) as usize;
    let address = ((address_upper_half << 8) | address_lower_half) << 4;
    let mp = check_mp_table(address as *const usize, 1024);
    if mp.is_some() {
        return mp.unwrap();
    }

    // Get last KB of the System Base Memory
    let address_upper_half = *bda.offset(0x14) as usize;
    let address_lower_half = *bda.offset(0x13) as usize;
    let address = ((address_upper_half << 8) | address_lower_half) * 1024;
    let mp = check_mp_table((address - 1024) as *const usize, 1024);
    if mp.is_some() {
        return mp.unwrap();
    }

    // Check everything in the BIOS ROM Memory
    let mp = check_mp_table(0xF0000 as *const usize, 0x10000);
    return mp.unwrap();
}

/// Explores the provided fragment of memory trying to find the MP Floating Pointer Structure based
/// on this signature, that is, _MP_.
unsafe fn check_mp_table(base: *const usize, length: isize) -> Option<*const MPFPStructure> {
    let mut base = P2V!(base as usize) as *const MPFPStructure;
    let end = base.byte_offset(length);

    loop {
        let current = base.as_ref().unwrap();
        if current.signature.as_slice() == b"_MP_" {
            // According to the MP specification, the MP table should have a checksum of 0
            let checksum = check_sum(base as *const u8, size_of::<MPFPStructure>());
            if checksum != 0 {
                panic!("[FATAL] Invalid MP Checksum");
            }

            return Some(base);
        }

        base = base.offset(1);
        if base >= end {
            break;
        }
    }

    return None;
}
