use alloc::string::{String, ToString};

use crate::{
    filesystem::fs::{find_inode_by_path, read_inode_data},
    memory::defs::{Page, KERNEL_BASE, PAGE_SIZE, PTE_P},
    println,
    scheduler::exec::{
        ELFHeader, ProgramHeader, ProgramHeaderType, ELF_HEADER_SIZE, ELF_MAGIC,
        ELF_PROG_HEADER_SIZE,
    },
    sync::cpu_cli::{pop_cli, push_cli},
    x86::helpers::{cli, load_cr3, read_cr3, sti},
    P2V, PTE_ADDRESS, PTE_FLAGS, V2P,
};

use super::vm::vm_corruption_checker;

pub fn check_process_memory(page_dir_address: usize) {
    push_cli();

    println!(
        "[DEBUG] Checking process with page dir at 0x{:X}",
        page_dir_address
    );

    let mut page_dir = Page::new(page_dir_address as *mut u8);
    let page_dir_entries = page_dir.cast_to::<usize>();
    let number_pages = KERNEL_BASE / (PAGE_SIZE * 1024);

    // Check all pages from 0 to KERNEL_BASE
    for i in 0..number_pages {
        let page_dir_entry = page_dir_entries[i];
        let is_present = PTE_FLAGS!(page_dir_entry) & PTE_P > 0;

        if is_present {
            println!("[DEBUG] Found Process Page Directory Entry ({i})");

            let page_table_address = P2V!(PTE_ADDRESS!(page_dir_entry));
            let mut page_table = Page::new(page_table_address as *mut u8);
            let page_table_entries = page_table.cast_to::<usize>();

            for j in 0..1024 {
                let page_table_entry = page_table_entries[j];
                let is_pt_present = PTE_FLAGS!(page_table_entry) & PTE_P > 0;

                if is_pt_present {
                    let virtual_address = i * 1024 * 4096 + j * 4096;
                    let physical_address = PTE_ADDRESS!(page_table_entry);

                    println!(
                        "[DEBUG] Address 0x{:X} is mapped at 0x{:X}",
                        virtual_address, physical_address
                    );
                }
            }
        }
    }

    let previous_cr3 = read_cr3();
    load_cr3(V2P!(page_dir_address));
    vm_corruption_checker();
    load_cr3(previous_cr3);

    pop_cli();
}

pub fn debug_elf(path: &str) {
    println!("[DEBUG] Checking ELF Binary");

    let inode = find_inode_by_path(path).unwrap();
    let mut data = read_inode_data(&inode, 0, ELF_HEADER_SIZE as u32);
    let header =
        unsafe { core::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut ELFHeader, 1)[0] };

    if header.magic != ELF_MAGIC {
        panic!("[FATAL, DEBUG] Not an ELF Binary");
    }

    println!("- ELF Magic: 0x{:X}", header.magic);
    println!("- ELF Type: {:?}", header._type);
    println!("- ELF Entry Point: 0x{:X}", header.entry);
    println!("- ELF Number Entries: {}", header.number_entries);
    println!("----- Program Headers -----");

    // Load program headers into memory
    let mut offset = header.program_header_offset as usize;
    for i in 0..header.number_entries {
        let data = read_inode_data(&inode, offset as u32, ELF_PROG_HEADER_SIZE as u32);
        let prog_header = unsafe { *(data.as_slice().as_ptr() as *const ProgramHeader) };
        offset += ELF_PROG_HEADER_SIZE;
        println!("----- Found Program Header [{}] -----", i);

        let prog_header_type_name = match prog_header._type as usize {
            0..=7 => prog_header._type,
            _ => ProgramHeaderType::OTHER,
        };
        println!(
            "- Program Header Type: 0x{:X} [{:?}]",
            prog_header._type as usize, prog_header_type_name
        );

        // Skip if this segment is not loadable
        if prog_header._type != ProgramHeaderType::LOAD {
            continue;
        }

        // An ELF whose memory size is less than the program data is invalid
        if prog_header.memory_size < prog_header.file_size {
            panic!("[PANIC, DEBUG] Memory Size is less than File Size");
        }

        let start_address = prog_header.virtual_address as usize;
        let end_address = start_address.wrapping_add(prog_header.memory_size as usize) as usize;
        if end_address < start_address {
            panic!("[PANIC, DEBUG] End Address is less than Start Address");
        }
    }
}
