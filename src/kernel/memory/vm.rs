use core::mem::size_of;
use spin::Mutex;

use super::{defs::*, gdt::GlobalDescriptorTable};
use crate::{kernel::misc::ds::LinkedList, println};

extern crate multiboot2;

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

pub static PAGE_LIST: Mutex<LinkedList<u64>> = Mutex::new(LinkedList::new());

fn zero_page(page_addr: u64) {
    unsafe {
        let page_addr = page_addr as *mut u64;
        for i in 0..(PAGE_SIZE) {
            (*page_addr.offset(i as isize)) = 0;
        }
    }
}

fn setup_mem_pages(mem_segment: (u64, u64)) -> u64 {
    let (addr, size) = mem_segment;
    let mem_top = addr + size;

    let rounded_base = pgroundup!(addr);
    let rounder_top = pgrounddown!(mem_top);
    let rounded_size = rounder_top - rounded_base;

    // let n_pages = (rounded_size / 4096) as u64;
    let n_pages = 5 as u64;

    let mut page_list = PAGE_LIST.lock();

    for i in 0..(n_pages) {
        (*page_list).push(rounded_base + i * 4096);
    }

    // println!("{:X}", (*page_list).pop().unwrap());
    // println!("{:X}", (*page_list).pop().unwrap());

    return n_pages;
}

fn free_page(page_addr: u64) {
    let mut page_list = PAGE_LIST.lock();
    zero_page(page_addr);

    (*page_list).push(page_addr);
}

fn get_free_page() -> Option<u64> {
    let mut page_list = PAGE_LIST.lock();
    return (*page_list).pop();
}

// Multiboot Memory Layout fields can be found at
// https://www.gnu.org/software/grub/manual/multiboot/html_node/multiboot_002eh.html
// Function responsible for indexing the memory layout and extracting the biggest memory
// segment.
fn setup_mem_layout(mem_pointer: usize) -> (u64, u64) {
    let boot_info_block = unsafe { multiboot2::load(mem_pointer) };
    let mem_tag = boot_info_block
        .memory_map_tag()
        .expect("Memory map tag not found");

    let mut memory_segment: (u64, u64) = (0, 0);
    for memory_area in mem_tag.memory_areas() {
        let (addr, size) = memory_segment;

        if (memory_area.length > size) {
            memory_segment = (memory_area.base_addr, memory_area.length);
        }
    }

    return memory_segment;
}

fn setup_gdt() {
    let mut gdt = GlobalDescriptorTable::new();

    // Kernel Code Segment
    gdt.create_segment(
        1,
        0x0,
        0xFFFFFFFF,
        GDT_RING0 | GDT_TYPE_E | GDT_TYPE_RW | GDT_TYPE_P | GDT_TYPE_S,
        GDT_FLAG_L,
    );
    // Kernel Data Segment
    gdt.create_segment(
        2,
        0x0,
        0xFFFFFFFF,
        GDT_RING0 | GDT_TYPE_RW | GDT_TYPE_P | GDT_TYPE_S,
        GDT_FLAG_L,
    );

    // User Code Segment
    gdt.create_segment(
        3,
        0x0,
        0xFFFFFFFF,
        GDT_RING3 | GDT_TYPE_E | GDT_TYPE_RW | GDT_TYPE_P | GDT_TYPE_S,
        GDT_FLAG_L,
    );
    // User Data Segment
    gdt.create_segment(
        4,
        0x0,
        0xFFFFFFFF,
        GDT_RING3 | GDT_TYPE_RW | GDT_TYPE_P | GDT_TYPE_S,
        GDT_FLAG_L,
    );

    // Update GDT
    gdt.set();
}

pub fn setup_vm(mem_layout_pointer: usize) {
    println!("[INIT] Mapping Memory");

    let mem_segment = setup_mem_layout(mem_layout_pointer);
    println!(
        "[INIT] Memory Segment From 0x{:X} to 0x{:X}",
        mem_segment.0,
        mem_segment.0 + mem_segment.1
    );

    let n_pages = setup_mem_pages(mem_segment);
    println!("[INIT] Mapped {} Pages", n_pages);

    setup_gdt();
    println!("[INIT] Global Description Page Updated");
}
