use core::{panic, slice::SliceIndex};

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
    P2V, PAGE_DIR_INDEX, PAGE_TABLE_INDEX, PTE_ADDRESS, ROUND_DOWN, V2P,
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
    pub static ref FREE_PAGE_LIST: SpinMutex<HeapLinkedList<usize>> =
        SpinMutex::new(HeapLinkedList::new());
}

/// Perform the allocation of pages. Gives priority to pages in the free list. If there
/// are no pages in the free list, then allocates from the static memory region. If no
/// pages are avaialable, raise an exception.
pub fn allocate_page<'a>() -> Page<'a> {
    if *IS_HEAP_ENABLED.lock() == true {
        let page = FREE_PAGE_LIST.lock().pop();

        match page {
            Some(page) => return Page::new(page as *mut u8),
            _ => (),
        }
    }

    let address = MEMORY_REGION.lock().next(1).expect("[FATAL] Out of Memory");
    Page::new(address)
}

pub fn allocate_pages<'a>(number_pages: usize) -> Page<'a> {
    let address = MEMORY_REGION
        .lock()
        .next(number_pages)
        .expect("[FATAL] Out of Memory");
    Page::new(address)
}

pub fn deallocate_page_dir(page_dir: &mut Page) {
    let page_dir_slice = page_dir.cast_to::<usize>();

    for i in 0..NUMBER_PAGE_ENTRIES {
        let page_entry = page_dir_slice[i * PAGE_ENTRY_SIZE];

        if page_entry & PTE_P > 0 {
            let page_address = P2V!(PTE_ADDRESS!(page_entry));
            deallocate_page(page_address);
        }
    }

    deallocate_page(page_dir.as_ptr() as usize);
}

/// Add pages to the Free List. The Free List can only be used if Heap is enabled.
pub fn deallocate_page(mut page_address: usize) {
    assert!(page_address % PAGE_SIZE == 0);

    if *IS_HEAP_ENABLED.lock() == false {
        panic!("[ERROR] Cannot dealocate without heap");
    }

    FREE_PAGE_LIST.lock().push(page_address);
}

/// Walk Page Directory uses the provided virtual memory address (virtual_address) to index
/// the page directory, and then the page table. If the page table is not present, allocates
/// a new page to act as the page table.
pub fn walk_page_dir(
    page_dir: &mut Page,
    virtual_address: usize,
    should_allocate: bool,
) -> *mut usize {
    let mut page_table: Page;

    let pg_dir_offset = PAGE_DIR_INDEX!(virtual_address as usize);
    let pg_dir_slice = page_dir.cast_to::<usize>();
    let pg_dir_entry = pg_dir_slice[pg_dir_offset];
    let is_entry_present = (pg_dir_entry & PTE_P) > 0;

    // If entry is already present, set page table simply as the address pointed by the entry
    if is_entry_present == true {
        page_table = Page::new(P2V!(pg_dir_entry & !0xFFF) as *mut u8);
    } else {
        // Since page was not found, we need to allocate
        if !should_allocate {
            panic!("Page walk failed: Not allowed to allocate");
        }

        // Create new entry in page directory
        page_table = allocate_page();
        page_table.zero();
        pg_dir_slice[pg_dir_offset] = V2P!(page_table.as_ptr() as usize) | PTE_P | PTE_W | PTE_U;
    }

    let page_table = page_table.cast_to::<usize>();
    let page_table_offset = PAGE_TABLE_INDEX!(virtual_address as usize);

    // Return the page entry that was found
    &mut page_table[page_table_offset] as *mut usize
}

/// Perform page mapping of a range into the provided page directory.
/// Creates all the necessary tables to accomodate pages from start_address
/// to end_address, starting for virtual_address.
pub fn map_pages(
    page_dir: &mut Page,
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
pub fn setup_kernel_page_tables<'a>() -> Result<Page<'a>, &'static str> {
    let mut page_dir: Page = allocate_page();
    page_dir.zero();

    let physical_top = unsafe { *PHYSICAL_TOP.lock() };

    if P2V!(physical_top) > DEVICE_SPACE {
        panic!("PHYSTOP is too high");
    }

    for entry in KERNEL_MEMORY_LAYOUT.lock().iter() {
        map_pages(
            &mut page_dir,
            entry.virt as usize,
            entry.phys_end.wrapping_sub(entry.phys_start),
            entry.phys_start,
            entry.perm,
        );
    }

    Ok(page_dir)
}

/// When the Kernel starts, it has only two pages defined. One page maps the physical
/// address and the other maps all addresses above KERNEL_BASE to the physical memory.
/// Here, we need to map the Kernel's memory layout to the one defined in KERNEL_MEMORY_LAYOUT,
/// ensuring access to the entirety of the physical space.
pub fn setup_vm() {
    println!("[KERNEL] Mapping Memory");

    let mut page_dir = setup_kernel_page_tables().expect("[ERR] Failed to Setup Virtual Memory");

    // Switch to new page directory
    let mut kernel_page_dir = KERNEL_PAGE_DIR.lock();
    *kernel_page_dir = Some(page_dir.as_ptr() as usize);
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
unsafe impl Send for Page<'_> {}

impl Page<'_> {
    pub fn new(address: *mut u8) -> Self {
        assert!(address as usize % PAGE_SIZE == 0);
        unsafe { Page(&mut *(address as *mut [u8; PAGE_SIZE])) }
    }

    pub fn as_ptr(&mut self) -> *mut u8 {
        self.0 as *mut u8
    }

    pub fn set_ptr(&mut self, address: *mut u8) {
        assert!(address as usize % PAGE_SIZE == 0);
        unsafe { self.0 = &mut *(address as *mut [u8; PAGE_SIZE]) };
    }

    // Call this function to ensure the page is zeroed before usage
    pub fn zero(&mut self) {
        memset(self.as_ptr(), 0, PAGE_SIZE);
    }

    pub fn cast_to<T>(&mut self) -> &mut [T] {
        let size = core::mem::size_of::<T>();
        let entries = PAGE_SIZE / size;
        unsafe { core::slice::from_raw_parts_mut(self.as_ptr() as *mut T, entries) }
    }
}

impl core::ops::Index<usize> for Page<'_> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl core::ops::IndexMut<usize> for Page<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl core::fmt::Display for Page<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
