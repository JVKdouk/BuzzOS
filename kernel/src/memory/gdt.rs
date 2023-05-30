use core::mem::size_of;

use crate::apic::mp::{get_my_cpu, CPUS};
use crate::memory::defs::*;
use crate::x86::helpers::load_cs;
use crate::{println, x86::helpers::lgdt};

impl TaskStateSegment {
    pub fn new() -> Self {
        TaskStateSegment::default()
    }

    pub fn get_segment(&self) -> u64 {
        let base_address = self as *const TaskStateSegment as u64;
        let tss_size = core::mem::size_of::<TaskStateSegment>() as u64 - 1;

        let base_0_23 = base_address & 0xFFFFFF;
        let base_24_31 = (base_address >> 24) & 0xFF;
        let limit_0_15 = tss_size & 0xFFFF;
        let limit_16_19 = (tss_size >> 16) & 0xF;
        let access_byte = 0x89;
        let flags = 0x0;

        return (base_24_31 << 56)
            | (flags << 52)
            | (limit_16_19 << 48)
            | (access_byte << 40)
            | (base_0_23 << 16)
            | limit_0_15;
    }
}

impl GlobalDescriptorTable {
    pub const fn new() -> Self {
        GlobalDescriptorTable([GlobalDescriptorTableSegment::new(); NUMBER_DESCRIPTORS])
    }

    pub fn set_segment(&mut self, index: u16, value: u64) {
        assert!(index < NUMBER_DESCRIPTORS as u16);

        self.0[index as usize].set(value);
    }

    pub fn set_long_segment(&mut self, index: u16, value: u128) {
        assert!(index + 1 < NUMBER_DESCRIPTORS as u16);

        self.0[index as usize].set(value as u64);
        self.0[(index + 1) as usize].set((value >> 64) as u64);
    }

    pub fn get_pointer(&self) -> u64 {
        let size = NUMBER_DESCRIPTORS * size_of::<u64>() - 1;
        let base = self.0.as_ptr() as u64;
        return (base << 16) | (size as u64);
    }

    pub fn refresh(&self) {
        // No worries of value being dropped here, since register is loaded before
        // the value is dropped.
        let pointer = self.get_pointer();
        lgdt(&pointer);

        // Code Segment Register must be refreshed with the GDT
        let cs_selector = self.get_selector(KERNEL_CODE_SEGMENT);
        load_cs(cs_selector);
    }

    pub fn get_selector(&self, index: u16) -> u16 {
        let descriptor = self.0[index as usize].0;
        let rpl = ((descriptor >> 45) & 0b11) as u16;
        (index << 3) | rpl
    }
}

impl GlobalDescriptorTableSegment {
    pub const fn new() -> Self {
        GlobalDescriptorTableSegment(0)
    }

    pub fn set(&mut self, value: u64) {
        self.0 = value;
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

impl DescriptorFlags {
    pub const KERNEL_CODE: u64 = Self::DEFAULT_SIZE.bits()
        | Self::NORMAL.bits()
        | Self::PRESENT.bits()
        | Self::WRITABLE.bits()
        | Self::EXECUTABLE.bits()
        | Self::GRANULARITY.bits()
        | Self::LIMIT_0_15.bits()
        | Self::LIMIT_16_19.bits();

    pub const KERNEL_DATA: u64 = Self::DEFAULT_SIZE.bits()
        | Self::NORMAL.bits()
        | Self::PRESENT.bits()
        | Self::WRITABLE.bits()
        | Self::GRANULARITY.bits()
        | Self::LIMIT_0_15.bits()
        | Self::LIMIT_16_19.bits();

    pub const USER_CODE: u64 = Self::KERNEL_CODE | Self::DPL_RING_3.bits();

    pub const USER_DATA: u64 = Self::KERNEL_DATA | Self::DPL_RING_3.bits();
}

pub fn setup_local_gdt() {
    let mut cpus = unsafe { CPUS.lock() };
    let mut cpu = get_my_cpu(&mut cpus).unwrap();
    let gdt = &mut cpu.gdt;

    // Setup TSS, used when switching between DPLs
    cpu.taskstate = Some(TaskStateSegment::new());
    let tss = cpu.taskstate.as_ref().unwrap().get_segment();

    gdt.set_segment(KERNEL_CODE_SEGMENT, DescriptorFlags::KERNEL_CODE);
    gdt.set_segment(KERNEL_DATA_SEGMENT, DescriptorFlags::KERNEL_DATA);
    gdt.set_segment(USER_CODE_SEGMENT, DescriptorFlags::USER_CODE);
    gdt.set_segment(USER_DATA_SEGMENT, DescriptorFlags::USER_DATA);
    gdt.set_long_segment(TASK_STATE_SEGMENT, tss as u128);

    cpu.gdt.refresh();
}

pub fn setup_gdt() {
    setup_local_gdt();
    println!("[KERNEL] CPU Descriptor Table Initialized ");
}
