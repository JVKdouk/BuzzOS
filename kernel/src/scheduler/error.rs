#[derive(Copy, Clone, Debug)]
pub enum ELFError {
    ELFOverflow(u32, u32),
    InvalidMemorySize(u32, u32),
    InvalidELFMagic(u32),
    KernelMappingFailure,
    MemoryAllocationFailure,
}

#[derive(Copy, Clone, Debug)]
pub enum ProcessError {
    SlotAllocationFailure,
    MemoryAllocationFailure,
}
