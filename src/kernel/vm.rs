use crate::println;
use super::defs;

pub fn setup_vm() {
    let memory_size = defs::MEMORY_TOP - defs::MEMORY_BOTTOM;
    let num_pages = memory_size / defs::MEMORY_PAGE_SIZE;
    println!("[INIT] {} Pages Mapped", num_pages);
}