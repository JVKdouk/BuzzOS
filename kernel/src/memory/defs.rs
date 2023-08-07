use core::mem::size_of;

use bitflags::bitflags;

use crate::structures::static_linked_list::StaticLinkedListNode;

/// Macros

/// Perform page rounding up, to the next page boundary
#[macro_export]
macro_rules! ROUND_UP {
    ($val:expr, $align:expr) => {
        ($val + $align - 1) & !($align - 1)
    };
}

/// Perform page rounding down, to the previous page boundary
#[macro_export]
macro_rules! ROUND_DOWN {
    ($val:expr, $align:expr) => {
        $val & !($align - 1)
    };
}

/// Convert memory address from virtual (above KERNEL_BASE) to physical (below KERNEL_BASE)
#[macro_export]
macro_rules! V2P {
    ($n:expr) => {
        ($n) - KERNEL_BASE
    };
}

/// Convert memory address from physical (below KERNEL_BASE) to virtual (above KERNEL_BASE)
#[macro_export]
macro_rules! P2V {
    ($n:expr) => {
        ($n) + KERNEL_BASE
    };
}

#[macro_export]
macro_rules! PAGE_TABLE_INDEX {
    ($n:expr) => {
        ($n >> PAGE_TABLE_SHIFT) & 0x3FF
    };
}

#[macro_export]
macro_rules! PAGE_DIR_INDEX {
    ($n:expr) => {
        ($n >> PAGE_DIR_SHIFT) & 0x3FF
    };
}

#[macro_export]
macro_rules! PTE_ADDRESS {
    ($n:expr) => {
        $n & !0xFFF
    };
}

/// GDT Definitions
pub const NUMBER_DESCRIPTORS: usize = 7;

pub const KERNEL_CODE_SEGMENT: u16 = 1;
pub const KERNEL_DATA_SEGMENT: u16 = 2;
pub const USER_CODE_SEGMENT: u16 = 3;
pub const USER_DATA_SEGMENT: u16 = 4;
pub const TASK_STATE_SEGMENT: u16 = 5;

/// Memory Layout
pub const MEM_BDA: usize = 0x400; // Memory BIOS Data Area

/// VM Definitions
pub const PAGE_SIZE: usize = 4096;
pub const PAGE_ENTRY_SIZE: usize = size_of::<u32>();
pub const NUMBER_PAGE_ENTRIES: usize = PAGE_SIZE / PAGE_ENTRY_SIZE;

pub const EXTENDED_MEMORY: usize = 0x100000;
pub const DEVICE_SPACE: usize = 0xFE000000;
pub const PHYSICAL_DEVICE_SPACE: usize = V2P!(DEVICE_SPACE);
pub const KERNEL_BASE: usize = 0x80000000;
pub const KERNEL_LINK: usize = KERNEL_BASE + EXTENDED_MEMORY;

pub const PAGE_DIR_SHIFT: usize = 22;
pub const PAGE_TABLE_SHIFT: usize = 12;

pub const PTE_P: usize = 0x001; // Present Bit
pub const PTE_W: usize = 0x002; // Writable Bit
pub const PTE_U: usize = 0x004; // User Bit
pub const PTE_PS: usize = 0x080; // Page Size Bit

/// Heap Definitions
pub const HEAP_PAGES: usize = 25;
pub const STACK_PAGES: usize = 4;

pub struct LinkedListAllocator {
    pub head: StaticLinkedListNode,
}

#[derive(Clone, Copy, Debug)]
pub struct GlobalDescriptorTableSegment(pub u64);

#[derive(Clone, Copy, Debug)]
pub struct GlobalDescriptorTable(pub [GlobalDescriptorTableSegment; NUMBER_DESCRIPTORS]);

#[derive(Debug)]
#[repr(C)]
pub struct MemoryLayoutEntry {
    pub virt: *const usize, // Start of the virtual address
    pub phys_start: usize,  // Start of the physical address
    pub phys_end: usize,    // End of the physical address
    pub perm: usize,        // Permission flags
}

#[derive(Debug)]
pub struct Page<'a>(pub &'a mut [u8; PAGE_SIZE]);

#[derive(Debug)]
pub struct MemoryRegion {
    pub start: usize,
    pub index: usize,
    pub end: usize,
}

bitflags! {
    pub struct DescriptorFlags: u64 {
        // Access
        const ACCESSED          = 1 << 40;
        const WRITABLE          = 1 << 41;
        const CONFORMING        = 1 << 42;
        const EXECUTABLE        = 1 << 43;
        const NORMAL            = 1 << 44;
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
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct TaskStateSegment {
    // Segment Selectors and Previous Task Link Field
    link: u32,
    pub esp0: u32,
    pub ss0: u16,
    reserved_0: u16,
    esp1: u32,
    ss1: u16,
    reserved_1: u16,
    esp2: u32,
    ss2: u16,
    reserved_2: u16,

    // Registers
    cr3: u32,
    eip: u32,
    eflags: u32,
    eax: u32,
    ecx: u32,
    edx: u32,
    ebx: u32,
    esp: u32,
    ebp: u32,
    esi: u32,
    edi: u32,

    // Selectors
    pub es: u16,
    reserved_5: u16,
    pub cs: u16,
    reserved_6: u16,
    pub ss: u16,
    reserved_7: u16,
    pub ds: u16,
    reserved_8: u16,
    pub fs: u16,
    reserved_9: u16,
    pub gs: u16,
    reserved_10: u16,
    ldtr: u16,
    reserved_11: u16,
    reserved_12: u16,

    // I/O Mapping
    pub iopb: u16,
}
