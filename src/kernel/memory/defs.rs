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
