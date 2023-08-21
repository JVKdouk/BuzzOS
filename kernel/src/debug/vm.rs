use core::slice::from_raw_parts_mut;

use crate::{
    memory::defs::{Page, KERNEL_BASE, PAGE_SIZE, PTE_P},
    println,
    x86::helpers::{cli, hlt, read_cr3},
    P2V, PTE_ADDRESS, PTE_FLAGS,
};

// Compares the memory of two page directories. While specific physical page addresses may not be equal
// the corresponding virtual memory location and data must.
pub unsafe fn compare_virtual_memory(first_pg_dir: &mut Page, second_pg_dir: &mut Page) {
    let page_entries = PAGE_SIZE / core::mem::size_of::<usize>();
    let first_page_dir_data = first_pg_dir.cast_to::<usize>();
    let second_page_dir_data = second_pg_dir.cast_to::<usize>();

    for i in 0..page_entries {
        if i * PAGE_SIZE.pow(2) > KERNEL_BASE {
            break;
        }

        let first_page_dir_entry = first_page_dir_data[i];
        let second_page_dir_entry = second_page_dir_data[i];

        let first_page_dir_flags = PTE_FLAGS!(first_page_dir_entry);
        let second_page_dir_flags = PTE_FLAGS!(second_page_dir_entry);

        // Check if flags are the same
        if first_page_dir_flags != second_page_dir_flags {
            panic!("[ERROR] VMs are different - Page Directory Flags");
        }

        // Empty page directory entry
        if first_page_dir_entry & PTE_P == 0 {
            continue;
        }

        // Avoid calling walk_page_dir here due to lookup slowdown
        let second_page_dir_entry = second_page_dir_data[i];
        let second_page_dir_address = P2V!(PTE_ADDRESS!(second_page_dir_entry)) as *mut usize;
        let second_page_table_data = from_raw_parts_mut(second_page_dir_address, PAGE_SIZE / 4);

        let first_page_dir_entry = first_page_dir_data[i];
        let first_page_dir_address = P2V!(PTE_ADDRESS!(first_page_dir_entry)) as *mut usize;
        let first_page_table_data = from_raw_parts_mut(first_page_dir_address, PAGE_SIZE / 4);

        for j in 0..page_entries {
            let first_page_table_entry_flags = PTE_FLAGS!(first_page_table_data[j]);
            let second_page_table_entry_flags = PTE_FLAGS!(second_page_table_data[j]);

            if second_page_table_entry_flags != first_page_table_entry_flags {
                panic!("[ERROR] VMs are different - Page Table Flags");
            }

            // Empty page table entry
            if first_page_table_data[j] & PTE_P == 0 {
                continue;
            }

            let first_page_entry = first_page_table_data[j];
            let first_page_address = P2V!(PTE_ADDRESS!(first_page_entry)) as *mut usize;
            let first_page_data = from_raw_parts_mut(first_page_address, PAGE_SIZE / 4);

            let second_page_entry = second_page_table_data[j];
            let second_page_address = P2V!(PTE_ADDRESS!(second_page_entry)) as *mut usize;
            let second_page_data = from_raw_parts_mut(second_page_address, PAGE_SIZE / 4);

            for k in 0..page_entries {
                if first_page_data[k] != second_page_data[k] {
                    panic!("[ERROR] VMs are different - Data Mismatch");
                }
            }
        }
    }

    println!("[SUCCESS] VMs are equal");
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
