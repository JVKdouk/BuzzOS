use core::panic;

use lazy_static::lazy_static;

use super::{
    defs::*,
    heap::IS_HEAP_ENABLED,
    mem::{MEMORY_REGION, PHYSICAL_TOP},
};
use crate::{
    memory::mem::memset,
    println,
    structures::heap_linked_list::HeapLinkedList,
    sync::spin_mutex::SpinMutex,
    x86::helpers::{cli, hlt, load_cr3, read_cr3},
    P2V, PAGE_DIR_INDEX, PAGE_TABLE_INDEX, ROUND_DOWN, ROUND_UP, V2P,
};

extern "C" {
    static KERNEL_DATA: u8;
}

lazy_static! {
    static ref KERNEL_MEMORY_LAYOUT: SpinMutex<[MemoryLayoutEntry; 4]> = SpinMutex::new([
        // I/O Address Space
        MemoryLayoutEntry {
            virt: KERNEL_BASE as *const usize,
            phys_start: 0,
            phys_end: EXTENDED_MEMORY,
            perm: PTE_W,
        },
        // Kernel Text + Read Only Data
        MemoryLayoutEntry {
            virt: KERNEL_LINK as *const usize,
            phys_start: V2P!(KERNEL_LINK),
            phys_end: unsafe { V2P!(&KERNEL_DATA as *const u8 as usize) },
            perm: 0,
        },
        // Kernel Data + Memory
        MemoryLayoutEntry {
            virt: unsafe { &KERNEL_DATA as *const u8 as *const usize },
            phys_start: unsafe { V2P!(&KERNEL_DATA as *const u8 as usize) },
            phys_end: unsafe { *PHYSICAL_TOP.lock() },
            perm: PTE_W,
        },
        // Other Devices
        MemoryLayoutEntry {
            virt: DEVICE_SPACE as *const usize,
            phys_start: DEVICE_SPACE,
            phys_end: 0,
            perm: PTE_W,
        },
    ]);
}

pub static KERNEL_PAGE_DIR: SpinMutex<Option<usize>> = SpinMutex::new(None);

lazy_static! {
    pub static ref FREE_PAGE_LIST: SpinMutex<HeapLinkedList<Page>> =
        SpinMutex::new(HeapLinkedList::new());
}

/// Perform the allocation of pages. Gives priority to pages in the free list. If there
/// are no pages in the free list, then allocates from the static memory region. If no
/// pages are avaialable, raise an exception.
pub fn allocate_page() -> Page {
    if *IS_HEAP_ENABLED.lock() == true {
        let page = FREE_PAGE_LIST.lock().pop();

        match page {
            Some(page) => return page,
            _ => (),
        }
    }

    return MEMORY_REGION.lock().next(1).expect("[FATAL] Out of Memory");
}

/// Add pages to the Free List. The Free List can only be used if Heap is enabled.
pub fn deallocate_page(page: Page) {
    assert_eq!(
        ROUND_UP!(page.address as usize, 4096),
        page.address as usize
    );

    if *IS_HEAP_ENABLED.lock() == false {
        panic!("[ERROR] Cannot dealocate without heap");
    }

    FREE_PAGE_LIST.lock().push(page);
}

/// Walk Page Directory uses the provided virtual memory address (virtual_address) to index
/// the page directory, and then the page table. If the page table is not present, allocates
/// a new page to act as the page table.
fn walk_page_dir(page_dir: Page, virtual_address: usize, should_allocate: bool) -> *mut usize {
    let page_directory_offset = PAGE_DIR_INDEX!(virtual_address as isize);
    let page_directory_entry = unsafe { *page_dir.address.offset(page_directory_offset) as usize };
    let is_entry_present = (page_directory_entry & PTE_P) > 0;

    let mut page_table: Page = Page {
        address: 0 as *const usize,
    };

    // If entry is already present, set page table simply as the address pointed by the entry
    if is_entry_present == true {
        page_table.address = P2V!(page_directory_entry & !0xFFF) as *const usize;
    } else {
        // Since page was not found, we need to allocate
        if !should_allocate {
            panic!("Page walk failed: Not allowed to allocate");
        }

        page_table = allocate_page();
        memset(page_table.address as usize, 0, PAGE_SIZE);

        unsafe {
            *(page_dir.address.offset(page_directory_offset) as *mut usize) =
                V2P!(page_table.address as usize) | PTE_P | PTE_W | PTE_U;
        }
    }

    let page_table_offset = PAGE_TABLE_INDEX!(virtual_address as isize);
    let page_table_entry = unsafe { (page_table.address as *mut usize).offset(page_table_offset) };

    // Return the page entry that was found
    page_table_entry
}

/// Perform page mapping of a range into the provided page directory.
/// Creates all the necessary tables to accomodate pages from start_address
/// to end_address, starting for virtual_address.
pub fn map_pages(
    page_dir: Page,
    virtual_address: usize,
    size: usize,
    mut physical_address: usize,
    perm: usize,
) {
    let mut start_address = ROUND_DOWN!(virtual_address, 4096) as *mut u8;
    let end_address =
        ROUND_DOWN!(virtual_address.wrapping_add(size).wrapping_sub(1), 4096) as *mut u8;

    loop {
        let page_table_entry = walk_page_dir(page_dir, start_address as usize, true);
        let is_page_entry_present = unsafe { *page_table_entry & PTE_P } > 0;

        // If the page is already mapped, then something went wrong
        if is_page_entry_present {
            panic!("[FATAL] Page was remapped");
        }

        // Map the page entry to the physical address
        unsafe { *page_table_entry = physical_address | perm | PTE_P }

        if start_address >= end_address {
            break;
        }

        start_address = unsafe { start_address.offset(PAGE_SIZE as isize) };
        physical_address += PAGE_SIZE;
    }
}

/// Maps each one of the entries of KERNEL_MEMORY_LAYOUT into a new page directory,
/// later switching CR3 to this new page directory.
pub fn setup_kernel_page_tables() -> Result<Page, &'static str> {
    let page_dir: Page = allocate_page();
    let physical_top = unsafe { *PHYSICAL_TOP.lock() };

    memset(page_dir.address as usize, 0, PAGE_SIZE);

    if P2V!(physical_top) > DEVICE_SPACE {
        panic!("PHYSTOP is too high");
    }

    for entry in KERNEL_MEMORY_LAYOUT.lock().iter() {
        // println!("[KERNEL] Page End at 0x{:X}", entry.phys_end);
        map_pages(
            page_dir,
            entry.virt as usize,
            entry.phys_end.wrapping_sub(entry.phys_start),
            entry.phys_start,
            entry.perm,
        );
    }

    return Ok(page_dir);
}

/// When the Kernel starts, it has only two pages defined. One page maps the physical
/// address and the other maps all addresses above KERNEL_BASE to the physical memory.
/// Here, we need to map the Kernel's memory layout to the one defined in KERNEL_MEMORY_LAYOUT,
/// ensuring access to the entirety of the physical space.
pub fn setup_vm() {
    println!("[KERNEL] Mapping Memory");

    let page_dir = setup_kernel_page_tables().expect("[ERR] Failed to Setup Virtual Memory");

    // Switch to new page directory
    let mut kernel_page_dir = KERNEL_PAGE_DIR.lock();
    *kernel_page_dir = Some(page_dir.address as usize);
    load_cr3(V2P!(kernel_page_dir.unwrap()));

    println!("[KERNEL] Virtual Memory Initialized");
}

#[no_mangle]
pub fn vm_corruption_checker() {
    cli();
    let cr3 = P2V!(read_cr3()) as *const u32;
    println!("Memory Check on Page Directory 0x{:X}", cr3 as u32);

    for pg_dir_index in 511..1024 {
        let page_dir_entry = unsafe { *cr3.add(pg_dir_index) };
        let page_table_address = page_dir_entry & 0xFFFFF000;
        let page_present = page_dir_entry & 0b1;
        let mut last_entry = 0;

        if page_present != 1 {
            continue;
        }

        for pg_table_index in 0..1024 {
            let entry_address = P2V!(page_table_address as usize) as *const u32;
            let entry_data = unsafe { *entry_address.add(pg_table_index) };
            let page_address = entry_data & 0xFFFFF000;

            if last_entry != 0 && page_address.overflowing_sub(last_entry).0 != 0x1000 {
                println!("[ERROR] Virtual Memory Corruption Detected");
                println!(
                    "[ERROR] First Corruption at 0x{:X}, Page Table Index {}, Page Directory Index {}",
                    page_table_address, pg_table_index, pg_dir_index
                );
                println!("[ERROR] 0x{:X} -> 0x{:X}", last_entry, page_address);
                hlt();
            }

            last_entry = page_address;
        }
    }

    println!("[KERNEL] Memory Check Completed. No Corruption Found");

    hlt();
}

unsafe impl Send for MemoryLayoutEntry {}
unsafe impl Send for Page {}
