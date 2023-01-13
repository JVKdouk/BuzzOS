use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;

use super::defs::*;
use crate::{misc::ds::LinkedList, println};

// External start of the data section
extern "C" {
    #[no_mangle]
    static data: *const u8;

    #[no_mangle]
    static edata: *const u8;

    #[no_mangle]
    static end: *const u8;
}

macro_rules! pgroundup {
    ($n:expr) => {
        $n + (4096 - ($n % 4096))
    };
}

macro_rules! pgrounddown {
    ($n:expr) => {
        $n - ($n % 4096)
    };
}

/// Convert memory address from virtual (above KERNEL_BASE) to physical (below KERNEL_BASE)
macro_rules! V2P {
    ($n:expr) => {
        ($n) - KERNEL_BASE
    };
}

/// Convert memory address from physical (below KERNEL_BASE) to virtual (above KERNEL_BASE)
macro_rules! P2V {
    ($n:expr) => {
        ($n) + KERNEL_BASE
    };
}

unsafe impl Send for MemoryLayoutEntry {}

lazy_static! {
    static ref KERNEL_MEMORY_LAYOUT: Mutex<[MemoryLayoutEntry; 4]> = Mutex::new([
    /// I/O Address Space
    MemoryLayoutEntry {
        virt: KERNEL_BASE as *const usize,
        phys_start: 0,
        phys_end: EXTENDED_MEMORY,
        perm: PTE_W,
    },
    /// Kernel Text + Read Only Data
    MemoryLayoutEntry {
        virt: KERNEL_LINK as *const usize,
        phys_start: V2P!(KERNEL_LINK),
        phys_end: unsafe { V2P!(data as usize) },
        perm: 0,
    },
    /// Kernel Data + Memory
    MemoryLayoutEntry {
        virt: unsafe { data as *const usize },
        phys_start: unsafe { V2P!(data as usize) },
        phys_end: 0x0, // Filled dynamically as the top of the physical memory
        perm: PTE_W,
    },
    /// Other Devices
    MemoryLayoutEntry {
        virt: DEVICE_SPACE as *const usize,
        phys_start: DEVICE_SPACE,
        phys_end: 0,
        perm: PTE_W,
    },
]);
}

// pub static PAGE_LIST: Mutex<LinkedList<u32>> = Mutex::new(LinkedList::new());

// fn setup_mem_pages(mem_segment: (u32, u32)) -> u32 {
//     let (addr, size) = mem_segment;
//     let mem_top = addr + size;

//     let rounded_base = pgroundup!(addr);
//     let rounder_top = pgrounddown!(mem_top);
//     let rounded_size = rounder_top - rounded_base;

//     // let n_pages = (rounded_size / 4096) as u32;
//     let n_pages = 5 as u32;

//     let mut page_list = PAGE_LIST.lock();

//     for i in 0..(n_pages) {
//         (*page_list).push(rounded_base + i * 4096);
//     }

//     // println!("{:X}", (*page_list).pop().unwrap());
//     // println!("{:X}", (*page_list).pop().unwrap());

//     return n_pages;
// }

fn zero_page(page_addr: u32) {
    unsafe {
        let page_addr = page_addr as *mut u32;
        for i in 0..(PAGE_SIZE) {
            (*page_addr.offset(i as isize)) = 0;
        }
    }
}

// fn free_page(page_addr: u32) {
//     if (page_addr % PAGE_SIZE || v < end)

//     let mut page_list = PAGE_LIST.lock();
//     zero_page(page_addr);

//     (*page_list).push(page_addr);
// }

// fn get_free_page() -> Option<u32> {
//     let mut page_list = PAGE_LIST.lock();
//     return (*page_list).pop();
// }

// // Multiboot Memory Layout fields can be found at
// // https://www.gnu.org/software/grub/manual/multiboot/html_node/multiboot_002eh.html
// // Function responsible for indexing the memory layout and extracting the biggest memory
// // segment.
// fn setup_mem_layout(mem_pointer: usize) -> (u32, u32) {
//     let boot_info_block = unsafe { multiboot2::load(mem_pointer) };
//     let mem_tag = boot_info_block
//         .memory_map_tag()
//         .expect("Memory map tag not found");

//     let mut memory_segment: (u32, u32) = (0, 0);
//     for memory_area in mem_tag.memory_areas() {
//         let (addr, size) = memory_segment;

//         if (memory_area.length > size) {
//             memory_segment = (memory_area.base_addr, memory_area.length);
//         }
//     }

//     return memory_segment;
// }

pub fn setup_vm() {
    println!("[INIT] {:p}", unsafe { end });
    println!("[INIT] Mapping Memory");

    // let mem_segment = setup_mem_layout(mem_layout_pointer);
    // println!(
    //     "[INIT] Memory Segment From 0x{:X} to 0x{:X}",
    //     mem_segment.0,
    //     mem_segment.0 + mem_segment.1
    // );

    // let n_pages = setup_mem_pages(mem_segment);
    println!("[INIT] Mapped {} Pages", 0);
}
