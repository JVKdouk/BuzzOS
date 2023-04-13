use core::mem::size_of;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::memory::defs::*;
use crate::x86::defs::ShortSegmentDescriptor;
use crate::x86::helpers::{load_cs, ltr};
use crate::{println, x86::helpers::lgdt};

use super::defs;

lazy_static! {
    pub static ref TSS: Mutex<TaskStateSegment> = {
        let mut tss = TaskStateSegment::new();
        Mutex::new(tss)
    };
    static ref GLOBAL_GDT: GlobalDescriptorTable = {
        let mut gdt = GlobalDescriptorTable::new();

        gdt.add_short_segment(KERNEL_CODE_SEGMENT);
        gdt.add_short_segment(KERNEL_DATA_SEGMENT);
        gdt.add_short_segment(USER_CODE_SEGMENT);
        gdt.add_short_segment(USER_DATA_SEGMENT);
        gdt.add_short_segment(TSS.lock().get_segment());

        gdt
    };
}

/// Global Descriptor Table is used in the process of segmentation. It is responsible
/// to convert the linear address into virtual address.
/// See more at https://en.wikipedia.org/wiki/Memory_segmentation
impl GlobalDescriptorTable {
    /// Create a new GDT
    pub const fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; defs::N_DESCRIPTORS],
            len: 1,
        }
    }

    /// Return the segment selector for a given index
    pub fn get_selector(&self, index: u16) -> u16 {
        let descriptor = self.table[index as usize];
        let rpl = ((descriptor >> 45) & 0b11) as u16;

        (index << 3) | rpl
    }

    /// Append a short segment (non-System) to the GDT
    pub fn add_short_segment(&mut self, segment: ShortSegmentDescriptor) {
        if self.len + 1 > self.table.len() {
            panic!("GDT is out of space");
        }

        self.add(segment);
    }

    /// Add a 64-bits value to the table
    fn add(&mut self, value: u64) {
        self.table[self.len] = value;
        self.len += 1;
    }

    /// Refresh GDT (using lgdt)
    pub fn refresh(&'static self) {
        unsafe { lgdt(&self.pointer()) };
    }

    /// Update Long Segment of the GDT
    pub fn set_segment(&mut self, index: u16, segment: ShortSegmentDescriptor) {
        if index as usize > self.table.len() - 1 {
            panic!("[FATAL] Index out of range");
        }

        self.table[index as usize] = segment as u64;
    }

    /// Return a pointer to the value formatted to fit the GDTR
    pub fn pointer(&self) -> GlobalDescriptorTablePointer {
        GlobalDescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            size: (self.len * size_of::<u64>() - 1) as u16,
        }
    }
}

impl DescriptorFlags {
    // A kernel data segment (64-bit or flat 32-bit)
    pub const KERNEL_DATA: Self = Self::from_bits_truncate(
        Self::DEFAULT_SIZE.bits()
            | Self::USER_SEGMENT.bits()
            | Self::DEFAULT_SIZE.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::LIMIT_0_15.bits()
            | Self::LIMIT_16_19.bits()
            | Self::GRANULARITY.bits(),
    );

    // A 64-bit kernel code segment
    pub const KERNEL_CODE32: Self = Self::from_bits_truncate(
        Self::USER_SEGMENT.bits()
            | Self::EXECUTABLE.bits()
            | Self::DEFAULT_SIZE.bits()
            | Self::PRESENT.bits()
            | Self::WRITABLE.bits()
            | Self::ACCESSED.bits()
            | Self::LIMIT_0_15.bits()
            | Self::LIMIT_16_19.bits()
            | Self::GRANULARITY.bits(),
    );

    // A user data segment (64-bit or flat 32-bit)
    pub const USER_DATA: Self =
        Self::from_bits_truncate(Self::KERNEL_DATA.bits() | Self::DPL_RING_3.bits());

    // A 64-bit user code segment
    pub const USER_CODE64: Self =
        Self::from_bits_truncate(Self::KERNEL_CODE32.bits() | Self::DPL_RING_3.bits());
}

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
        let flags = 0x4;

        return (base_24_31 << 56)
            | (flags << 52)
            | (limit_16_19 << 48)
            | (access_byte << 40)
            | (base_0_23 << 16)
            | limit_0_15;
    }
}

pub fn setup_gdt() {
    GLOBAL_GDT.refresh();
    let cs_selector = GLOBAL_GDT.get_selector(1);
    load_cs(cs_selector);

    println!("[KERNEL] Global Descriptor Table Initialized ");
}
