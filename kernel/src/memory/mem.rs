/// Implementation of memory related utilities. Notice Virtual Memory are located in vm.rs.
/// Here, you can find the implementation of the Memory Region, an iterator that statically
/// maps the entirety of the Memory Region (from where the Kernel finishes being linked to the
/// top of physical memory) into pages. By doing it this way, we don't have to map all pages to
/// the free list at boot time, only when a page is freed.
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{print, println, x86::helpers::stosb, P2V, ROUND_UP};

use super::defs::{MemoryRegion, Page, KERNEL_BASE};

extern "C" {
    static KERNEL_END: u8;
}

pub static mut PHYSICAL_TOP: Mutex<usize> = Mutex::new(0xE000000);

lazy_static! {
    pub static ref MEMORY_REGION: Mutex<MemoryRegion> = {
        let start = unsafe { ROUND_UP!(&KERNEL_END as *const u8 as usize, 4096) };
        let end = P2V!(unsafe { *PHYSICAL_TOP.lock() });
        Mutex::new(MemoryRegion::new(start, end))
    };
}

pub fn memset(address: usize, value: u8, length: usize) {
    stosb(address, value, length);
}

pub unsafe fn memmove(src: usize, dst: usize, mut length: usize) -> *mut usize {
    let mut src = src as *mut u8;
    let mut dst = dst as *mut u8;

    if src < dst && src.offset(length as isize) > dst {
        src = src.offset(length as isize);
        dst = src.offset(length as isize);

        while length > 0 {
            length -= 1;
            dst = dst.offset(-1);
            src = src.offset(-1);
            *dst = *src;
        }
    } else {
        while length > 0 {
            length -= 1;
            *dst = *src;
            dst = dst.offset(1);
            src = src.offset(1);
        }
    }

    return dst as *mut usize;
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
    /// raises an exception.
    pub fn next(&mut self, number_pages: usize) -> Result<Page, &'static str> {
        if self.index as usize + 4096 * number_pages > self.end {
            return Err("[ERR] Failure to Allocate Page");
        }

        let address_index_offset = (self.index * 4096) as usize;
        let address = self.start + address_index_offset;
        self.index += number_pages;

        Ok(Page {
            address: address as *const usize,
        })
    }
}

unsafe impl Send for MemoryRegion {}
