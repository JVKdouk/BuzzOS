use bitflags::bitflags;

// Number of entries in a page (Used for PML4, PDP, PD, and PT)
pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;

// Page table entry flags. You can find more at https://wiki.osdev.org/Paging#Page_Table
bitflags! {
    pub struct PageTableFlags: u64 {
        const PRESENT = 1;              // Bit 0  - Is it present in memory?
        const WRITABLE = 1 << 1;        // Bit 1  - Is it writable?
        const USER_ACCESSIBLE = 1 << 2; // Bit 2  - Is it user accessible (Ring 3)?
        const WRITE_THROUGH = 1 << 3;   // Bit 3  - Use write-through caching policy?
        const NO_CACHE = 1 << 4;        // Bit 4  - Should use cache?
        const ACCESSED = 1 << 5;        // Bit 5  - Was it accessed recently? (Use for Swapping)
        const DIRTY = 1 << 6;           // Bit 6  - Is it dirty (Written)?
        const HUGE_PAGE = 1 << 7;       // Bit 7  - Is it a huge page (2MiB)?
        const GLOBAL = 1 << 8;          // Bit 8  - Is it present in all address-spaces? (Not flushed by TLB)
        const BIT_9 = 1 << 9;           // Bit 9  - Unused. Can be used for Custom Flags
        const BIT_10 = 1 << 10;         // Bit 10 - Unused. Can be used for Custom Flags
        const BIT_11 = 1 << 11;         // Bit 11 - Unused. Can be used for Custom Flags
        const BIT_52 = 1 << 52;         // Bit 52 - Unused. Can be used for Custom Flags
        const BIT_53 = 1 << 53;         // Bit 53 - Unused. Can be used for Custom Flags
        const BIT_54 = 1 << 54;         // Bit 54 - Unused. Can be used for Custom Flags
        const BIT_55 = 1 << 55;         // Bit 55 - Unused. Can be used for Custom Flags
        const BIT_56 = 1 << 56;         // Bit 56 - Unused. Can be used for Custom Flags
        const BIT_57 = 1 << 57;         // Bit 57 - Unused. Can be used for Custom Flags
        const BIT_58 = 1 << 58;         // Bit 58 - Unused. Can be used for Custom Flags
        const BIT_59 = 1 << 59;         // Bit 59 - Unused. Can be used for Custom Flags
        const BIT_60 = 1 << 60;         // Bit 60 - Unused. Can be used for Custom Flags
        const BIT_61 = 1 << 61;         // Bit 61 - Unused. Can be used for Custom Flags
        const BIT_62 = 1 << 62;         // Bit 62 - Unused. Can be used for Custom Flags
        const NO_EXECUTE = 1 << 63;     // Bit 63 - Is it executable?
    }
}

/// A 64-bit page table entry.
#[derive(Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT],
}

#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct PageDirectory {
    entries: [PageTableEntry; PAGE_TABLE_ENTRY_COUNT],
}
