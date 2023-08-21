/// Implementation of memory related utilities. Notice Virtual Memory are located in vm.rs.
/// Here, you can find the implementation of the Memory Region, an iterator that statically
/// maps the entirety of the Memory Region (from where the Kernel finishes being linked to the
/// top of physical memory) into pages. By doing it this way, we don't have to map all pages to
/// the free list at boot time, only when a page is freed.
use lazy_static::lazy_static;

use crate::{sync::spin_mutex::SpinMutex, x86::helpers::stosb, P2V, ROUND_UP};

use super::{
    defs::{MemoryRegion, KERNEL_BASE, PAGE_SIZE},
    error::MemoryError,
};

extern "C" {
    static KERNEL_END: u8;
}

pub static mut PHYSICAL_TOP: SpinMutex<usize> = SpinMutex::new(0xE000000);

lazy_static! {
    pub static ref MEMORY_REGION: SpinMutex<MemoryRegion> = {
        let start = unsafe { ROUND_UP!(&KERNEL_END as *const u8 as usize, 4096) };
        let end = P2V!(unsafe { *PHYSICAL_TOP.lock() });
        SpinMutex::new(MemoryRegion::new(start, end))
    };
}

pub fn mem_set(address: *mut u8, value: u8, length: usize) {
    stosb(address as usize, value, length);
}

pub unsafe fn mem_move(mut src: *mut u8, mut dst: *mut u8, mut length: usize) {
    while length > 0 {
        length -= 1;
        *dst = *src;
        dst = dst.offset(1);
        src = src.offset(1);
    }
}

/// Defines a memory region from start (pointer) to end (integer). This is used
/// in the initial Kernel loading procedure. Since all pages from V2P!(KERNEL_END) to
/// PHYSICAL_TOP are guaranteed to be unused by the linker, we can allocate new pages
/// from that region. Once pages are freed, they can be added to the free list.
impl MemoryRegion {
    pub fn new(start: usize, end: usize) -> Self {
        MemoryRegion {
            start: ROUND_UP!(start, 4096),
            index: 0,
            end,
        }
    }

    /// Gets the next page available and increase the counter. Once no more pages are available,
    /// it raises an OutOfMemory exception.
    pub fn next(&mut self, number_pages: usize) -> Result<*mut u8, MemoryError> {
        if self.index as usize + PAGE_SIZE * number_pages > self.end {
            return Err(MemoryError::OutOfMemory);
        }

        let address_index_offset = (self.index * 4096) as usize;
        let address = self.start + address_index_offset;
        self.index += number_pages;

        Ok(address as *mut u8)
    }
}

unsafe impl Send for MemoryRegion {}
