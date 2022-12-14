use bitflags::bitflags;

pub const MEMORY_TOP: u64 = 0xFFFFFFFFFFFFFFFF;
pub const MEMORY_BOTTOM: u64 = 0x0;
pub const MEMORY_PAGE_SIZE: u64 = 4096;

pub const N_DESCRIPTORS: usize = 6;

pub const GDT_FLAG_L: u8 = 0x2;
pub const GDT_FLAG_DB: u8 = 0x4;
pub const GDT_FLAG_G: u8 = 0x8;

pub const GDT_TYPE_A: u8 = 0x1;
pub const GDT_TYPE_RW: u8 = 0x2;
pub const GDT_TYPE_DC: u8 = 0x4;
pub const GDT_TYPE_E: u8 = 0x8;
pub const GDT_RING0: u8 = 0x00;
pub const GDT_RING1: u8 = 0x20;
pub const GDT_RING2: u8 = 0x40;
pub const GDT_RING3: u8 = 0x60;
pub const GDT_TYPE_S: u8 = 0x10;
pub const GDT_TYPE_P: u8 = 0x80;

pub const PAGE_SIZE: u16 = 4096;

#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable {
    pub table: [u64; N_DESCRIPTORS], // Segment Descriptor List
    pub len: usize,                  // Size of GDT
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct GlobalDescriptorTablePointer {
    pub size: u16,
    pub base: u64,
}

bitflags! {
    pub struct DescriptorFlags: u64 {
        // Access
        const ACCESSED          = 1 << 40;
        const WRITABLE          = 1 << 41;
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const USER_SEGMENT      = 1 << 44;
        const DPL_RING_3        = 3 << 45;
        const PRESENT           = 1 << 47;
        const AVAILABLE         = 1 << 52;

        // Flags
        const LONG_MODE         = 1 << 53;
        const DEFAULT_SIZE      = 1 << 54;
        const GRANULARITY       = 1 << 55;

        // Limit
        const LIMIT_0_15        = 0xFFFF;
        const LIMIT_16_19       = 0xF << 48;

        // Base
        const BASE_0_23         = 0xFF_FFFF << 16;
        const BASE_24_31        = 0xFF << 56;
    }
}

/// Common segments
pub const KERNEL_CODE_SEGMENT: u64 = DescriptorFlags::KERNEL_CODE64.bits();
pub const KERNEL_DATA_SEGMENT: u64 = DescriptorFlags::KERNEL_DATA.bits();
pub const USER_CODE_SEGMENT: u64 = DescriptorFlags::USER_CODE64.bits();
pub const USER_DATA_SEGMENT: u64 = DescriptorFlags::USER_DATA.bits();
