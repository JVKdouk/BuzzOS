use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;

use super::defs::*;
use crate::{misc::ds::LinkedList, println};

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

// pub static PAGE_LIST: Mutex<LinkedList<u64>> = Mutex::new(LinkedList::new());

// fn zero_page(page_addr: u64) {
//     unsafe {
//         let page_addr = page_addr as *mut u64;
//         for i in 0..(PAGE_SIZE) {
//             (*page_addr.offset(i as isize)) = 0;
//         }
//     }
// }

// fn setup_mem_pages(mem_segment: (u64, u64)) -> u64 {
//     let (addr, size) = mem_segment;
//     let mem_top = addr + size;

//     let rounded_base = pgroundup!(addr);
//     let rounder_top = pgrounddown!(mem_top);
//     let rounded_size = rounder_top - rounded_base;

//     // let n_pages = (rounded_size / 4096) as u64;
//     let n_pages = 5 as u64;

//     let mut page_list = PAGE_LIST.lock();

//     for i in 0..(n_pages) {
//         (*page_list).push(rounded_base + i * 4096);
//     }

//     // println!("{:X}", (*page_list).pop().unwrap());
//     // println!("{:X}", (*page_list).pop().unwrap());

//     return n_pages;
// }

// fn free_page(page_addr: u64) {
//     let mut page_list = PAGE_LIST.lock();
//     zero_page(page_addr);

//     (*page_list).push(page_addr);
// }

// fn get_free_page() -> Option<u64> {
//     let mut page_list = PAGE_LIST.lock();
//     return (*page_list).pop();
// }

// // Multiboot Memory Layout fields can be found at
// // https://www.gnu.org/software/grub/manual/multiboot/html_node/multiboot_002eh.html
// // Function responsible for indexing the memory layout and extracting the biggest memory
// // segment.
// fn setup_mem_layout(mem_pointer: usize) -> (u64, u64) {
//     let boot_info_block = unsafe { multiboot2::load(mem_pointer) };
//     let mem_tag = boot_info_block
//         .memory_map_tag()
//         .expect("Memory map tag not found");

//     let mut memory_segment: (u64, u64) = (0, 0);
//     for memory_area in mem_tag.memory_areas() {
//         let (addr, size) = memory_segment;

//         if (memory_area.length > size) {
//             memory_segment = (memory_area.base_addr, memory_area.length);
//         }
//     }

//     return memory_segment;
// }

pub fn setup_vm(mem_layout_pointer: usize) {
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
