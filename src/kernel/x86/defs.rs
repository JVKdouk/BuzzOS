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

// Definition of a segment selector. Facilitates construction of segment address (GDT Offset + Privilege)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SegmentSelector(pub u16);

impl SegmentSelector {
    #[inline]
    pub const fn new(index: u16, priv_level: PrivilegeLevel) -> SegmentSelector {
        SegmentSelector(index << 3 | (priv_level as u16))
    }

    /// Returns the GDT index.
    #[inline]
    pub fn get_index(self) -> u16 {
        self.0 >> 3
    }

    /// Returns the requested privilege level.
    #[inline]
    pub fn get_priv_level(self) -> PrivilegeLevel {
        match (self.0 & 0b11) {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            i => panic!("privilege level {} is invalid", i),
        }
    }
}

// Segmentation. Most of segmentation is disabled in 64 bits due to paging. However, Segment
// Selectors are still used. CS, SS, and DS are just a few. They index into the
// the GDT, and operate as offsets to memory accesses.
pub trait Segment {
    fn get_reg() -> SegmentSelector;
    unsafe fn set_reg(sel: SegmentSelector);
}

#[derive(Debug)]
pub struct CS;
