// ***************** Security *****************

/// Protection Ring
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PrivilegeLevel {
    Ring0 = 0, // Kernel Level
    Ring1 = 1, // Some Device Drivers
    Ring2 = 2, // General Purpose
    Ring3 = 3, // User Level
}

// *************** Segmentation ***************

pub type ShortSegmentDescriptor = u64;
pub type LongSegmentDescriptor = u128;

// Segmentation. Most of segmentation is disabled in 64 bits due to paging. However, Segment
// Selectors are still used. CS, SS, and DS are just a few. They index into the
// the GDT, and operate as offsets to memory accesses.
pub trait Segment {
    fn get_reg() -> u16;
    unsafe fn set_reg(sel: u16);
}

#[derive(Debug)]
pub struct CS;
