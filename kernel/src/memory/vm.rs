use core::mem::size_of;
use lazy_static::lazy_static;
use spin::Mutex;

use super::{defs::*, mem};
use crate::{
    memory::mem::memset, pgrounddown, pgroundup, print, println, x86::helpers::lcr3, P2V,
    PAGE_DIR_INDEX, PAGE_TABLE_INDEX, V2P,
};

// External start of the data section
extern "C" {
    #[no_mangle]
    static KERNEL_END: u8;

    #[no_mangle]
    static data: u8;
}

unsafe impl Send for MemoryLayoutEntry {}

lazy_static! {
    static ref KERNEL_MEMORY_LAYOUT: Mutex<[MemoryLayoutEntry; 4]> = Mutex::new([
    /// I/O Address Space (256 pages)
    MemoryLayoutEntry {
        virt: KERNEL_BASE as *const usize,
        phys_start: 0,
        phys_end: EXTENDED_MEMORY,
        perm: PTE_W,
    },
    /// Kernel Text + Read Only Data (9 pages)
    MemoryLayoutEntry {
        virt: KERNEL_LINK as *const usize,
        phys_start: V2P!(KERNEL_LINK),
        phys_end: unsafe { V2P!(&data as *const u8 as usize) },
        perm: 0,
    },
    /// Kernel Data + Memory (57059 pages)
    MemoryLayoutEntry {
        virt: unsafe { &data as *const u8 as *const usize },
        phys_start: unsafe { V2P!(&data as *const u8 as usize) },
        phys_end: PHYSICAL_TOP, // Filled dynamically as the top of the physical memory
        perm: PTE_W,
    },
    /// Other Devices (?? pages)
    MemoryLayoutEntry {
        virt: DEVICE_SPACE as *const usize,
        phys_start: DEVICE_SPACE,
        phys_end: 0,
        perm: PTE_W,
    },
]);
}

pub static KERNEL_PAGE_DIR: Mutex<Option<usize>> = Mutex::new(None);

lazy_static! {
    pub static ref MEMORY_REGION: Mutex<MemoryRegion> = Mutex::new(MemoryRegion::new(
        unsafe { &KERNEL_END },
        P2V!(PHYSICAL_TOP)
    ));
}

fn allocate_page() -> Result<Page, &'static str> {
    return MEMORY_REGION.lock().next();
}

fn walk_page_dir(
    page_dir: Page,
    virtual_address: usize,
    should_allocate: bool,
) -> Result<*mut usize, &'static str> {
    let page_directory_offset = PAGE_DIR_INDEX!(virtual_address as isize);
    let page_directory_entry = unsafe { *page_dir.address.offset(page_directory_offset) as usize };
    let is_entry_valid = (page_directory_entry & PTE_P) > 0;

    let mut page_table: Page = Page {
        address: 0 as *const usize,
    };

    // If entry is already present, set page table simply as the address pointed by the entry
    if (is_entry_valid == true) {
        page_table.address = (page_directory_entry & !0xFFF) as *const usize;
    } else {
        // Since page was not found, we need to allocate
        if (!should_allocate) {
            return Err("Page walk failed: Not allowed to allocate");
        }

        page_table = allocate_page()?;
        memset(page_table.address as usize, 0, PAGE_SIZE);

        unsafe {
            *(page_dir.address.offset(page_directory_offset) as *mut usize) =
                V2P!(page_table.address as usize) | PTE_P | PTE_W | PTE_U;
        }
    }

    let page_table_offset = PAGE_TABLE_INDEX!(virtual_address as isize);
    let page_table_entry = unsafe { (page_table.address as *mut usize).offset(page_table_offset) };
    return Ok(page_table_entry);
}

/// Perform page mapping into the provided page directory.
fn map_pages(
    page_dir: Page,
    virtual_address: usize,
    size: usize,
    mut physical_address: usize,
    perm: usize,
) -> Result<(), &'static str> {
    let mut start_address = pgrounddown!(virtual_address) as *mut u8;
    let end_address = pgrounddown!(virtual_address.wrapping_add(size).wrapping_sub(1)) as *mut u8;

    let mut count = 0;

    loop {
        let page_table_entry = walk_page_dir(page_dir, start_address as usize, true)?;

        let is_page_entry_present = unsafe { *page_table_entry & PTE_P } > 0;

        if (is_page_entry_present) {
            return Err("Page table was remapped");
        }

        unsafe { *page_table_entry = physical_address | perm | PTE_P }

        if (start_address >= end_address) {
            break;
        }

        start_address = unsafe { start_address.offset(PAGE_SIZE as isize) };
        physical_address += PAGE_SIZE;
        count += 1;
    }

    return Ok(());
}

fn switch_kernel_vm(virtual_address: usize) {
    lcr3(V2P!(virtual_address));
}

/// Maps each one of the entries of KERNEL_MEMORY_LAYOUT into a new page directory,
/// later switching CR3 to this new page directory.
fn setup_kernel_page_tables() -> Result<Page, &'static str> {
    let page_dir: Page = allocate_page()?;

    memset(page_dir.address as usize, 0, PAGE_SIZE);

    if (P2V!(PHYSICAL_TOP) > DEVICE_SPACE) {
        panic!("PHYSTOP is too high");
    }

    for entry in KERNEL_MEMORY_LAYOUT.lock().iter() {
        map_pages(
            page_dir,
            entry.virt as usize,
            entry.phys_end.wrapping_sub(entry.phys_start),
            entry.phys_start,
            entry.perm,
        )?;
    }

    let mut kernel_page_dir = KERNEL_PAGE_DIR.lock();
    *kernel_page_dir = Some(page_dir.address as usize);
    switch_kernel_vm(kernel_page_dir.unwrap());

    return Ok(page_dir);
}

/// When the Kernel starts, it has only two pages defined. One page maps the physical
/// address and the other maps all addresses above KERNEL_BASE to the physical memory.
/// Here, we need to map the Kernel's memory layout to the one defined in KERNEL_MEMORY_LAYOUT,
/// ensuring access to the entirety of the physical space.
pub fn setup_vm() {
    println!("[KERNEL] Mapping Memory");
    setup_kernel_page_tables();
    println!("[KERNEL] Virtual Memory Initialized");
}
