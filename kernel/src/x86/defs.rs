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
