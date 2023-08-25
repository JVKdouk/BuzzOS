#[derive(Copy, Clone, Debug)]
pub enum MemoryError {
    // System used up all available memory
    OutOfMemory,

    // Heap has not yet been allocated
    HeapUnavailable,

    // Process memory tries to invade Kernel memory space (KERNEL_BASE)
    MemorySpaceViolation,

    // Physical Top must obey some rules, including not being above Device Space
    InvalidPhysicalTop(u32),

    // Page not found on page walk
    PageNotFound(u32),

    // Page already present when a map is called
    PageRemapped(u32),
}
