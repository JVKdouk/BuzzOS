use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;

use super::defs::*;
use crate::{println, x86::helpers::lcr3};

// External start of the data section
extern "C" {
    #[no_mangle]
    static data: *const u8;

    #[no_mangle]
    static edata: *const u8;

    #[no_mangle]
    static end: usize;
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

macro_rules! PAGE_TABLE_INDEX {
    ($n:expr) => {
        ($n >> PAGE_TABLE_SHIFT) & 0x3FF
    };
}

macro_rules! PAGE_DIR_INDEX {
    ($n:expr) => {
        ($n >> PAGE_DIR_SHIFT) & 0x3FF
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

pub static KERNEL_PAGE_DIR: Mutex<Option<usize>> = Mutex::new(None);
pub static PAGE_LIST: Mutex<[usize; 8192]> = Mutex::new([0; 8192]);

fn free_range(start_address: usize, end_address: usize) {
    let start_address = pgroundup!(start_address);

    for page_address in ((start_address + PAGE_SIZE)..=end_address).step_by(PAGE_SIZE) {
        free_page(page_address);
    }
}

fn zero_page(page_addr: usize) {
    unsafe {
        let page_addr = page_addr as *mut u32;
        for i in 0..(PAGE_SIZE) {
            (*page_addr.offset(i as isize)) = 0;
        }
    }
}

fn free_page(page_addr: usize) {
    let is_aligned = page_addr % PAGE_SIZE as usize == 0;
    let is_in_range = page_addr >= unsafe { end } && V2P!(page_addr) < PHYSICAL_TOP;

    if (!is_aligned || !is_in_range) {
        panic!("Could not free page");
    }

    let mut page_list = PAGE_LIST.lock();
    zero_page(page_addr);
}

fn allocate_page() -> usize {
    let mut page_list = PAGE_LIST.lock();
    let value = 1;
    return value;
}

fn walk_page_dir(
    page_dir: *mut usize,
    virtual_address: usize,
    should_allocate: bool,
) -> *mut usize {
    let page_directory_offset = PAGE_DIR_INDEX!(virtual_address as isize);
    let page_directory_entry = unsafe { *page_dir.offset(page_directory_offset) };
    let is_entry_valid = (page_directory_entry & PTE_P) > 0;

    let page_table: usize;

    if (is_entry_valid == true) {
        page_table = page_directory_entry & !0xFFF;
    } else {
        if (!should_allocate) {
            panic!("Page walk failed: Not allowed to allocate");
        }

        page_table = allocate_page();
        zero_page(page_table);

        unsafe {
            *(page_directory_entry as *mut usize) = V2P!(page_table) | PTE_P | PTE_W | PTE_U;
        }
    }

    let page_table_offset = PAGE_TABLE_INDEX!(virtual_address as isize);
    return unsafe { (page_table as *mut usize).offset(page_table_offset) };
}

fn map_pages(
    page_dir: *mut usize,
    virtual_address: usize,
    size: usize,
    mut physical_address: usize,
    perm: usize,
) {
    let mut start_address = pgrounddown!(virtual_address) as *mut usize;
    let end_address = pgrounddown!(virtual_address + size - 1) as *mut usize;

    loop {
        let page_table_entry = walk_page_dir(page_dir, virtual_address, true);
        let is_page_entry_present = unsafe { *page_table_entry & PTE_P } > 0;

        if (is_page_entry_present) {
            panic!("Page table was remapped");
        }

        unsafe { *page_table_entry = physical_address | perm | PTE_P }

        if (start_address == end_address) {
            break;
        }

        start_address = unsafe { start_address.offset(PAGE_SIZE as isize) };
        physical_address += PAGE_SIZE;
    }
}

fn setup_kernel_page_tables() -> *mut usize {
    let page_dir: *mut usize = allocate_page() as *mut usize;

    zero_page(page_dir as usize);

    if (P2V!(PHYSICAL_TOP) > DEVICE_SPACE) {
        panic!("PHYSTOP is too high");
    }

    let kernel_mem_layout = KERNEL_MEMORY_LAYOUT.lock();

    for entry in kernel_mem_layout.iter() {
        map_pages(
            page_dir,
            entry.virt as usize,
            entry.phys_end - entry.phys_start,
            entry.phys_start,
            entry.perm,
        );
    }

    return page_dir;
}

fn switch_kernel_vm() {
    lcr3(KERNEL_PAGE_DIR.lock().unwrap());
}

pub fn setup_vm() {
    println!("[INIT] Mapping Memory");
    println!("[INIT] Mapped {} Pages", 0);
}
