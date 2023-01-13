use bitflags::bitflags;

// Number of entries in a page (Used for PML4, PDP, PD, and PT)
pub const PAGE_TABLE_ENTRY_COUNT: usize = 512;

// Page table entry flags. You can find more at https://wiki.osdev.org/Paging#Page_Table
bitflags! {
    pub struct PageTableFlags: u32 {
        const PRESENT = 1;              // Bit 0  - Is it present in memory?
        const WRITABLE = 1 << 1;        // Bit 1  - Is it writable?
        const USER_ACCESSIBLE = 1 << 2; // Bit 2  - Is it user accessible (Ring 3)?
        const WRITE_THROUGH = 1 << 3;   // Bit 3  - Use write-through caching policy?
        const NO_CACHE = 1 << 4;        // Bit 4  - Should use cache?
        const ACCESSED = 1 << 5;        // Bit 5  - Was it accessed recently? (Use for Swapping)
        const DIRTY = 1 << 6;           // Bit 6  - Is it dirty (Written)?
        const HUGE_PAGE = 1 << 7;       // Bit 7  - Is it a huge page (2MiB)?
        const GLOBAL = 1 << 8;          // Bit 8  - Is it present in all address-spaces? (Not flushed by TLB)
        const BIT_9 = 1 << 9;           // Bit 9 - Available
        const BIT_10 = 1 << 9;          // Bit 10 - Available
        const BIT_11 = 1 << 9;          // Bit 11 - Available
    }
}

/// A 64-bit page table entry.
#[derive(Clone)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u32,
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
