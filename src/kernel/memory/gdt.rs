use core::mem::size_of;

use crate::kernel::x86::helpers::lgdt;

use super::defs;

// Segment Descriptor Fields
const LIM_15_0: u32 = 0xFFFF;
const BASE_15_0: u32 = 0xFFFF;
const BASE_23_16: u32 = 0xFF0000;
const TYPE: u8 = 0xFF;
const FLAGS: u8 = 0xF;
const LIM_19_16: u32 = 0xFF0000;
const BASE_31_24: u32 = 0xFF000000;

#[derive(Debug, Clone)]
pub struct GlobalDescriptorTable {
    table: [u64; defs::N_DESCRIPTORS], // Segment Descriptor List
    len: usize,                        // Size of GDT
}

pub struct GlobalDescriptorTablePointer {
    base: u64,
    size: u16,
}

impl GlobalDescriptorTable {
    #[inline]
    pub const fn new() -> GlobalDescriptorTable {
        GlobalDescriptorTable {
            table: [0; defs::N_DESCRIPTORS],
            len: 1,
        }
    }

    pub fn create_segment(&mut self, i: usize, base: u32, limit: u32, _type: u8, flags: u8) {
        let base_15_0 = (base & BASE_15_0) as u64;
        let base_23_16 = ((base & BASE_23_16) >> 16) as u64;
        let base_31_24 = ((base & BASE_31_24) >> 24) as u64;

        let lim_15_0 = (base & LIM_15_0) as u64;
        let lim_19_16 = ((base & LIM_19_16) >> 16) as u64;

        let type_7_0 = (_type & TYPE) as u64;
        let flags_3_0 = (flags & FLAGS) as u64;

        let segment: u64 = (base_31_24 << 56)
            | (flags_3_0 << 52)
            | (lim_19_16 << 48)
            | (type_7_0 << 40)
            | (base_23_16 << 32)
            | (base_15_0 << 16)
            | lim_15_0;

        self.table[i] = segment;
    }

    pub fn set(&self) {
        unsafe {
            lgdt(&self.pointer());
        }
    }

    pub fn pointer(&self) -> GlobalDescriptorTablePointer {
        return GlobalDescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            size: (self.len * size_of::<u64>() - 1) as u16,
        };
    }
}
