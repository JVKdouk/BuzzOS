use alloc::string::String;

use crate::{
    filesystem::fs::{read_inode_data, INode},
    memory::{
        defs::{Page, KERNEL_BASE, PAGE_SIZE, PTE_U, PTE_W},
        vm::{setup_kernel_page_tables, walk_page_dir},
    },
    scheduler::process::allocate_range,
    P2V, PTE_ADDRESS, ROUND_UP,
};

use super::{error::ELFError, process::load_process_memory, scheduler::SCHEDULER};

const ELF_MAGIC: u32 = 0x464C457F;
const ELF_HEADER_SIZE: usize = core::mem::size_of::<ELFHeader>();
const ELF_PROG_HEADER_SIZE: usize = core::mem::size_of::<ProgramHeader>();

const DEFAULT_PROGRAM_STACK_SIZE: usize = 8192; // Stack size in bytes

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum ProgramHeaderType {
    NULL = 0,
    LOAD = 1,
    DYNAMIC = 2,
    INTERPRETER = 3,
    NOTE = 4,
    SHLIB = 5,
    HEADER = 6,
    THREAD = 7,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum ELFHeaderType {
    NULL = 0,
    RELOCATABLE = 1,
    EXECUTABLE = 2,
    DYNAMIC = 3,
    CORE = 4,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ELFHeader {
    magic: u32,
    specs: [u8; 12],
    _type: ELFHeaderType,
    machine: u16,
    version: u32,
    entry: u32, // Program Entry Point
    program_header_offset: u32,
    section_header_offset: u32,
    flags: u32,
    header_size: u16,
    header_table_size: u16,
    number_entries: u16, // Number of entries in the ELF
    section_entry_size: u16,
    section_header_number: u16,
    section_names_index: u16,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ProgramHeader {
    _type: ProgramHeaderType,
    offset: u32,
    virtual_address: u32,
    physical_address: u32,
    file_size: u32,
    memory_size: u32,
    flags: u32,
    align: u32,
}

fn get_elf_header(inode: &INode) -> ELFHeader {
    // Read ELF Header of the inode
    let mut data = read_inode_data(&inode, 0, ELF_HEADER_SIZE as u32);
    unsafe { core::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut ELFHeader, 1)[0] }
}

fn read_program_header(inode: &INode, offset: u32) -> ProgramHeader {
    // Read program header block of the inode at offset
    let data = read_inode_data(&inode, offset, ELF_PROG_HEADER_SIZE as u32);

    // Convert data into a Program Header
    unsafe { *(data.as_slice().as_ptr() as *const ProgramHeader) }
}

/// Allocate the user stack alongside one guard-page to detect stack overflow
pub fn prepare_stack(page_dir: &mut Page, address: usize) -> Result<usize, ELFError> {
    let address = ROUND_UP!(address, PAGE_SIZE);
    let stack_size = ROUND_UP!(DEFAULT_PROGRAM_STACK_SIZE, PAGE_SIZE) / PAGE_SIZE + 1;

    let Ok(_) = allocate_range(page_dir, address, address + stack_size * PAGE_SIZE) else {
        return Err(ELFError::MemoryAllocationFailure);
    };

    // This is a guard page, used to detect stack overflows, since it cannot be writen by the user
    let page_table_entry = walk_page_dir(page_dir, address, false).unwrap();
    unsafe { *page_table_entry = *page_table_entry & !PTE_U & !PTE_W };

    let last_page_index = address + (stack_size - 1) * PAGE_SIZE;
    let page_table_entry = walk_page_dir(page_dir, last_page_index, false).unwrap();
    let page_table_address = unsafe { PTE_ADDRESS!(P2V!(*page_table_entry)) };
    let mut page = Page::new(page_table_address as *mut u8);

    let page_data = page.cast_to::<usize>();
    let mut esp = address + stack_size * PAGE_SIZE; // Move ESP to top of the stack

    page_data[PAGE_SIZE / 4 - 1] = 0xAAAAAAAA;
    page_data[PAGE_SIZE / 4 - 2] = 2;
    page_data[PAGE_SIZE / 4 - 3] = 0xFFFFFFFF; // Return trap

    esp -= 3 * core::mem::size_of::<usize>();
    Ok(esp)
}

pub fn decode_elf(inode: &INode) -> Result<(Page, ELFHeader, usize), ELFError> {
    // Check if this is an ELF Executable. No support to other formats yet
    let header = get_elf_header(inode);

    if header.magic != ELF_MAGIC {
        return Err(ELFError::InvalidELFMagic(header.magic));
    }

    let Ok(mut page_dir) = setup_kernel_page_tables() else {
        return Err(ELFError::KernelMappingFailure);
    };

    // Load program headers into memory
    let mut highest_page_address = 0;
    let mut offset = header.program_header_offset as usize;
    for _ in 0..header.number_entries {
        let prog_header = read_program_header(inode, offset as u32);
        offset += ELF_PROG_HEADER_SIZE;

        // Skip if this segment is not loadable
        if prog_header._type != ProgramHeaderType::LOAD {
            continue;
        }

        // An ELF whose memory size is less than the program data is invalid
        if prog_header.memory_size < prog_header.file_size {
            return Err(ELFError::InvalidMemorySize(
                prog_header.memory_size,
                prog_header.file_size,
            ));
        }

        let start_address = prog_header.virtual_address as usize;
        let end_address = start_address.wrapping_add(prog_header.memory_size as usize) as usize;
        if end_address < start_address {
            return Err(ELFError::ELFOverflow(
                start_address as u32,
                end_address as u32,
            ));
        }

        // Allocate all required pages for this section to be loaded into memory
        let Ok(_) = allocate_range(&mut page_dir, start_address, end_address) else {
            return Err(ELFError::MemoryAllocationFailure);
        };

        if end_address > highest_page_address {
            highest_page_address = end_address;
        }

        // Finally, load the program code into memory
        load_process_memory(
            &mut page_dir,
            start_address as *const u8,
            inode,
            prog_header.offset as usize,
            prog_header.file_size as usize,
        )
        .unwrap();
    }

    Ok((page_dir, header, highest_page_address))
}

pub fn exec(inode: &INode, name: String) {
    let mut scheduler = unsafe { SCHEDULER.lock() };
    let process = scheduler.current_process.as_ref().unwrap();

    let (mut new_page_dir, header, highest_page_address) =
        decode_elf(inode).expect("[ERROR] Failed to decode ELF");

    // Update process's page directory
    process.lock().pgdir = Some(new_page_dir.as_mut_ptr() as *mut usize);
    process.lock().name = name;

    // Prepare process stack page
    let Ok(esp) = prepare_stack(&mut new_page_dir, highest_page_address) else {
        panic!("[FATAL] Execution Failed");
    };

    unsafe { (*process.lock().trapframe.unwrap()).esp = esp };
    unsafe { (*process.lock().trapframe.unwrap()).eip = header.entry as usize };

    // Return to scheduler
    unsafe { scheduler.resume() };
}
