use core::{panic, sync::atomic::Ordering};

use lazy_static::lazy_static;

use super::{
    defs::*,
    error::MemoryError,
    heap::IS_HEAP_ENABLED,
    mem::{mem_set, MEMORY_REGION, PHYSICAL_TOP},
};

use crate::{
    println,
    scheduler::{self, scheduler::SCHEDULER},
    structures::heap_linked_list::HeapLinkedList,
    sync::spin_mutex::SpinMutex,
    x86::helpers::load_cr3,
    P2V, PAGE_DIR_INDEX, PAGE_TABLE_INDEX, PTE_ADDRESS, ROUND_DOWN, V2P,
};

extern "C" {
    // Entry point to kernel data + free memory space
    static KERNEL_DATA: u8;
}

// Entirety of Kernel Virtual Memory. Those are segments of memory meant to be loaded by the
// Setup Kernel Virtual Memory procedure, allowing certain portions to be written to and some to only be read.
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
            phys_end: PHYSICAL_TOP.load(Ordering::Relaxed),
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
pub static FREE_PAGE_LIST: SpinMutex<HeapLinkedList<usize>> = SpinMutex::new(HeapLinkedList::new());

/// Perform the allocation of pages. Gives priority to pages in the free list. If there
/// are no pages in the free list, then allocates from the static memory region. If no
/// pages are avaialable, raise an exception.
pub fn allocate_page<'a>() -> Result<Page<'a>, MemoryError> {
    if IS_HEAP_ENABLED.load(Ordering::Relaxed) {
        match FREE_PAGE_LIST.lock().pop() {
            Some(address) => return Ok(Page::new(address as *mut u8)),
            None => (),
        };
    }

    let address = MEMORY_REGION.lock().next(1)?;
    return Ok(Page::new(address));
}

pub fn allocate_pages<'a>(number_pages: usize) -> Result<Page<'a>, MemoryError> {
    let address = MEMORY_REGION.lock().next(number_pages)?;
    Ok(Page::new(address))
}

/// TODO: Finish this
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
pub fn deallocate_page(page_address: usize) {
    assert!(page_address % PAGE_SIZE == 0);

    if IS_HEAP_ENABLED.load(Ordering::Relaxed) == false {
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
) -> Result<*mut usize, MemoryError> {
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
            return Err(MemoryError::PageNotFound(virtual_address as u32));
        }

        // Create new entry in page directory
        page_table = allocate_page()?;
        page_table.zero();

        pg_dir_slice[pg_dir_offset] = V2P!(page_table.as_ptr() as usize) | PTE_P | PTE_W | PTE_U;
    }

    // Find page table entry
    let page_table_data = page_table.cast_to::<usize>();
    let page_table_offset = PAGE_TABLE_INDEX!(virtual_address as usize);
    let page_table_entry = &mut page_table_data[page_table_offset];

    // Return the page table entry pointer
    Ok(page_table_entry as *mut usize)
}

/// Map a continuous range of physical pages into the provided page directory, starting at
/// virtual_memory and ending at virtual_memory + size. Returns a pointer to the starting address
/// if allocation goes as expected, else returns a memory error.
pub fn map_pages(
    page_dir: &mut Page,
    virtual_address: usize,
    size: usize,
    mut physical_address: usize,
    perm: usize,
) -> Result<*mut u8, MemoryError> {
    assert!(size > 0);

    let start_address = ROUND_DOWN!(virtual_address, 4096) as *mut u8;
    let end_address = ROUND_DOWN!(virtual_address.wrapping_add(size - 1), 4096) as *mut u8;
    let mut current_address = start_address;

    loop {
        let page_table_entry = walk_page_dir(page_dir, current_address as usize, true)?;
        let is_page_entry_present = unsafe { *page_table_entry & PTE_P } > 0;

        // If the page is already mapped, then something went wrong
        if is_page_entry_present {
            return Err(MemoryError::PageRemapped(current_address as u32));
        }

        // Map the page entry to the physical address
        unsafe { *page_table_entry = physical_address | perm | PTE_P }

        if current_address >= end_address {
            break;
        }

        current_address = unsafe { current_address.offset(PAGE_SIZE as isize) };
        physical_address += PAGE_SIZE;
    }

    Ok(start_address)
}

/// Maps each one of the entries of KERNEL_MEMORY_LAYOUT into a new page directory,
/// later switching CR3 to this new page directory.
pub fn setup_kernel_page_tables<'a>() -> Result<Page<'a>, MemoryError> {
    let mut page_dir: Page = allocate_page().expect("[ERROR] Failed to allocate page");
    page_dir.zero();

    // Physical top cannot be above the device space
    let physical_top = PHYSICAL_TOP.load(Ordering::Relaxed);
    if P2V!(physical_top) > DEVICE_SPACE {
        return Err(MemoryError::InvalidPhysicalTop(physical_top as u32));
    }

    // Map each memory region as defined in the Kernel Memory Layout
    for entry in KERNEL_MEMORY_LAYOUT.lock().iter() {
        map_pages(
            &mut page_dir,
            entry.virt as usize,
            entry.phys_end.wrapping_sub(entry.phys_start),
            entry.phys_start,
            entry.perm,
        )?;
    }

    Ok(page_dir)
}

/// When the Kernel starts, it has only two pages defined. One page maps the physical
/// address and the other maps all addresses above KERNEL_BASE to the physical memory.
/// Here, we need to map the Kernel's memory layout to the one defined in KERNEL_MEMORY_LAYOUT,
/// ensuring access to the entirety of the physical space.
pub fn setup_vm() {
    let page_dir = setup_kernel_page_tables().expect("[ERR] Failed to Setup Virtual Memory");
    let page_dir_ptr = page_dir.as_ptr();

    // Switch to new page directory
    let mut kernel_page_dir = KERNEL_PAGE_DIR.lock();
    *kernel_page_dir = Some(page_dir_ptr as usize);
    load_cr3(V2P!(page_dir_ptr as usize));

    println!("[KERNEL] Virtual Memory Initialized");
}

unsafe impl Send for MemoryLayoutEntry {}
unsafe impl Send for Page<'_> {}

impl Page<'_> {
    pub fn new(address: *mut u8) -> Self {
        assert!(address as usize % PAGE_SIZE == 0);
        unsafe { Page(&mut *(address as *mut [u8; PAGE_SIZE])) }
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.0 as *mut u8
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.0.as_ref().as_ptr()
    }

    pub fn set_ptr(&mut self, address: *mut u8) {
        assert!(address as usize % PAGE_SIZE == 0);
        unsafe { self.0 = &mut *(address as *mut [u8; PAGE_SIZE]) };
    }

    // Call this function to ensure the page is zeroed before usage
    pub fn zero(&mut self) {
        mem_set(self.as_mut_ptr(), 0, PAGE_SIZE);
    }

    pub fn cast_to<T>(&mut self) -> &mut [T] {
        let size = core::mem::size_of::<T>();
        let entries = PAGE_SIZE / size;
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr() as *mut T, entries) }
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
