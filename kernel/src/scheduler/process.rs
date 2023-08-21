use core::{panic, slice::from_raw_parts_mut};

use alloc::{string::String, sync::Arc, vec::Vec};

use super::{
    defs::process::{Context, Process, ProcessList, ProcessState, TrapFrame},
    scheduler::{PROCESS_LIST, SCHEDULER},
    sleep::sleep,
};

use crate::{
    apic::mp::get_my_cpu,
    debug::vm::compare_virtual_memory,
    filesystem::fs::{read_inode_data, INode},
    memory::{
        defs::{
            Page, KERNEL_BASE, KERNEL_DATA_SEGMENT, PAGE_SIZE, PTE_P, PTE_U, PTE_W,
            TASK_STATE_SEGMENT, USER_CODE_SEGMENT, USER_DATA_SEGMENT,
        },
        error::MemoryError,
        mem::{mem_move, mem_set},
        vm::{allocate_page, map_pages, setup_kernel_page_tables, walk_page_dir},
    },
    sync::{
        cpu_cli::{pop_cli, push_cli},
        spin_mutex::{SpinMutex, SpinMutexGuard},
    },
    x86::{
        defs::PrivilegeLevel,
        helpers::{load_cr3, ltr},
    },
    P2V, PTE_ADDRESS, PTE_FLAGS, ROUND_DOWN, ROUND_UP, V2P,
};

impl ProcessList {
    pub const fn new() -> Self {
        ProcessList {
            list: Vec::new(),
            next_to_visit: 0,
            next_pid: 0,
        }
    }

    pub fn get_pid(&self, pid: usize) -> Option<Arc<SpinMutex<Process>>> {
        if pid > self.list.len() {
            panic!("[FATAL] PID out of bounds");
        }

        match self.list.iter().find(|process| process.lock().pid == pid) {
            Some(process) => Some(Arc::clone(process)),
            None => None,
        }
    }

    /// Allocation of process requires an empty slot. Finds the next empty slot
    /// to allocate the process.
    pub fn insert_process(&mut self, mut process: Process) -> Option<usize> {
        match self
            .list
            .iter()
            .position(|p| p.lock().state == ProcessState::EMPTY)
        {
            Some(pid) => {
                process.pid = pid;
                self.list[pid] = Arc::new(SpinMutex::new(process));
                Some(pid)
            }
            None => {
                process.pid = self.list.len();
                self.list.push(Arc::new(SpinMutex::new(process)));
                Some(self.list.len() - 1)
            }
        }
    }

    pub fn get_next_ready(&mut self) -> Option<&Arc<SpinMutex<Process>>> {
        if self.next_to_visit >= self.list.len() {
            self.next_to_visit = 0;
        }

        let mut index = self.next_to_visit;
        for process in self.list.iter().skip(self.next_to_visit) {
            if process.lock().state == ProcessState::READY {
                break;
            }

            index += 1;
        }

        self.next_to_visit = index + 1;
        if index >= self.list.len() {
            return None;
        }

        Some(&self.list[index])
    }
}

impl Process {
    pub fn new(pid: usize) -> Self {
        Process {
            state: ProcessState::EMPTY,
            mem_size: Default::default(),
            current_working_directory: String::from("/"),
            name: String::from(""),
            context: None,
            trapframe: None,
            kernel_stack: None,
            pgdir: None,
            sleep_object: 0,
            parent: None,
            pid,
        }
    }

    pub fn set_trapframe(&mut self, trapframe: TrapFrame) {
        unsafe {
            // (*self.trapframe.unwrap()).eax = trapframe.eax;
            *self.trapframe.unwrap() = trapframe;
        }
    }

    pub fn get_trapframe(&self) -> Option<TrapFrame> {
        match self.trapframe {
            Some(trapframe) => unsafe { Some(*trapframe) },
            None => None,
        }
    }
}

extern "C" {
    static _binary_init_start: usize;
    static _binary_init_size: usize;

    pub fn trap_return();
}

/// Spawn a process block. Notice the process block has no meaning until it is queued to be run
/// by the Scheduler. At this point, it is generally prepared to run in the user-space, but no
/// specific are provided.
pub unsafe fn spawn_process() -> Result<usize, &'static str> {
    let mut process_list = PROCESS_LIST.lock();
    let mut process = Process::new(process_list.next_pid);
    process_list.next_pid += 1;

    let trapframe_size = core::mem::size_of::<TrapFrame>() as isize;
    let context_size = core::mem::size_of::<Context>() as isize;
    let mut kernel_page = allocate_page().expect("[ERROR] Failed to allocate page");
    let mut kernel_page_pointer = kernel_page.as_mut_ptr() as *mut usize;
    let mut esp = kernel_page_pointer.offset(PAGE_SIZE as isize / 4);

    process.kernel_stack = Some(kernel_page_pointer);
    process.mem_size = PAGE_SIZE;
    process.state = ProcessState::EMBRYO;

    // Setup Trapframe Layout
    esp = esp.offset(-trapframe_size / 4);
    mem_set(esp as *mut u8, 0, trapframe_size as usize);
    process.trapframe = Some(esp as *mut TrapFrame);

    // Setup Context Layout
    esp = esp.offset(-context_size / 4);
    mem_set(esp as *mut u8, 0, context_size as usize);
    process.context = Some(esp as *mut Context);

    // Create Trap Return
    (*process.context.unwrap()).eip = trap_return as *const () as usize;

    // Return the pid for the newly created process slot
    match process_list.insert_process(process) {
        Some(pid) => Ok(pid),
        None => Err("Could not allocate"),
    }
}

unsafe fn set_user_tss(process: &SpinMutexGuard<Process>) {
    let cpu = get_my_cpu().unwrap();
    let mut task_lock = cpu.taskstate.lock();
    let mut task_state = task_lock.as_mut().unwrap();
    let gdt = &mut cpu.gdt.lock();

    gdt.set_long_segment(TASK_STATE_SEGMENT, task_state.get_segment() as u128);
    task_state.esp0 = process.kernel_stack.unwrap() as u32 + PAGE_SIZE as u32;
    task_state.ss0 = (KERNEL_DATA_SEGMENT << 3) as u16;
    task_state.iopb = 0xFFFF;

    ltr((TASK_STATE_SEGMENT as u16) << 3);
}

/// Switch to User Virtual Memory, departing from Kernel Virtual Memory. Notice this does
/// not yet change the DPL or CPL, as this is done at a later step. Since Kernel memory is
/// entirely mapped for every process, the process can continue executing after the memory is
/// switched.
pub unsafe fn switch_user_virtual_memory(process: &Arc<SpinMutex<Process>>) {
    let process_lock = process.lock();
    let page_dir = process_lock
        .pgdir
        .expect("[FATAL] Process has no page directory") as usize;

    // TODO: Switch to push-cli and pop-cli
    push_cli();
    set_user_tss(&process_lock);
    load_cr3(V2P!(page_dir));
    pop_cli();
}

/// Migrate from Kernel Virtual Memory to User Virtual Memory. Notice that
pub unsafe fn setup_user_virtual_memory(page_dir: &mut Page, address: *const usize, size: usize) {
    if size >= PAGE_SIZE {
        panic!("[FATAL] User Virtual Memory is bigger than one page");
    }

    let mut memory_page = allocate_page().expect("[ERROR] Failed to allocate page");
    let virtual_address = 0;
    let page_size = PAGE_SIZE;
    let phys_address = V2P!(memory_page.as_ptr() as usize);
    let flags = PTE_W | PTE_U;

    // Prepare all pages required by the process and copy over the executable binary and data
    memory_page.zero();
    mem_move(address as *mut u8, memory_page.as_mut_ptr(), size);
    map_pages(page_dir, virtual_address, page_size, phys_address, flags)
        .expect("[ERROR] Failed to Map Kernel Pages");
}

/// Spawns the first process to run in the user space, the init process. Subsequent children inherit
/// many attributes of the init process, such as trapframe,
pub unsafe fn spawn_init_process() {
    let process_pid = spawn_process().expect("[FATAL] Failed to start init process");
    let mut process = PROCESS_LIST.lock().get_pid(process_pid).unwrap();
    let mut process_lock = process.lock();

    let mut kernel_pgdir =
        setup_kernel_page_tables().expect("[FATAL] Failed to setup Kernel pages");

    let user_code_selector = (USER_CODE_SEGMENT << 3) as u16 | PrivilegeLevel::Ring3 as u16;
    let user_data_selector = (USER_DATA_SEGMENT << 3) as u16 | PrivilegeLevel::Ring3 as u16;

    process_lock.pgdir = Some(kernel_pgdir.as_mut_ptr() as *mut usize);

    setup_user_virtual_memory(
        &mut kernel_pgdir,
        &_binary_init_start as *const usize,
        &_binary_init_size as *const usize as usize,
    );

    // Setup Trapframe
    (*process_lock.trapframe.unwrap()).esp = PAGE_SIZE;
    (*process_lock.trapframe.unwrap()).eip = 0;
    (*process_lock.trapframe.unwrap()).cs = user_code_selector;
    (*process_lock.trapframe.unwrap()).ds = user_data_selector;
    (*process_lock.trapframe.unwrap()).es = user_data_selector;
    (*process_lock.trapframe.unwrap()).ss = user_data_selector;
    (*process_lock.trapframe.unwrap()).eflags = 0x200;

    // Setup Misc
    process_lock.name = String::from("kernel_init");
    process_lock.state = ProcessState::READY;
}

/// TODO: Use Map Pages instead of allocate range
/// Perform virtual memory mapping on a given range. Useful to map multiple pages at once.
pub fn allocate_range(page_dir: &mut Page, mut start_address: usize, end_address: usize) {
    start_address = ROUND_DOWN!(start_address, PAGE_SIZE);

    while start_address < end_address {
        let mut page = allocate_page().expect("[ERROR] Failed to allocate page");
        page.zero();

        // Map page into a process's virtual memory
        map_pages(
            page_dir,
            start_address,
            PAGE_SIZE,
            V2P!(page.as_ptr() as usize),
            PTE_W | PTE_U,
        );

        start_address += PAGE_SIZE;
    }
}

pub fn load_process_memory(
    page_dir: &mut Page,
    address: *const u8,
    inode: &INode,
    offset: usize,
    size: usize,
) -> Result<(), ()> {
    if offset > PAGE_SIZE {
        panic!("[ERROR] ELF Offset bigger than a full page");
    }

    let mut counter = 0;
    let mut first_page_offset = offset;
    while counter < size {
        // Get page that will be modified
        let page_table_entry = walk_page_dir(page_dir, address as usize + counter, false).unwrap();
        let page_address = unsafe { P2V!(*page_table_entry & !0xFFF) };

        // Align to minimum between what data is left and a full page
        let mut byte_count = PAGE_SIZE;
        if size - counter < PAGE_SIZE {
            byte_count = size - counter;
        }

        // Read program's data
        let inode_data = read_inode_data(inode, (offset + counter) as u32, byte_count as u32);

        // Write data into the page
        let page_data = unsafe { from_raw_parts_mut(page_address as *mut u8, PAGE_SIZE) };
        page_data[first_page_offset..(first_page_offset + byte_count)]
            .copy_from_slice(inode_data.as_slice());

        counter += PAGE_SIZE;
        first_page_offset = 0;
    }

    Ok(())
}

pub unsafe fn resize_process_memory(
    page_dir: &mut Page,
    mut old_size: usize,
    new_size: usize,
) -> Result<usize, MemoryError> {
    // TODO: Add support to shrinking
    if new_size < old_size {
        return Ok(old_size);
    }

    if new_size > KERNEL_BASE {
        return Err(MemoryError::MemorySpaceViolation);
    }

    let mut lower_boundary = ROUND_UP!(old_size, PAGE_SIZE);
    while lower_boundary < new_size {
        // Allocate new pages to the current process
        let mut page = allocate_page()?;
        page.zero();

        // Map page into a process's virtual memory
        map_pages(
            page_dir,
            lower_boundary,
            PAGE_SIZE,
            V2P!(page.as_ptr() as usize),
            PTE_W | PTE_U,
        )?;

        lower_boundary += PAGE_SIZE;
    }

    Ok(new_size)
}

pub fn fork() {
    let new_process_pid = unsafe { spawn_process().unwrap() };
    let mut new_process = unsafe { PROCESS_LIST.lock().get_pid(new_process_pid).unwrap() };
    let mut kernel_pgdir = setup_kernel_page_tables().unwrap();
    new_process.lock().pgdir = Some(kernel_pgdir.as_mut_ptr() as *mut usize);

    let mut scheduler = unsafe { SCHEDULER.lock() };
    let process = scheduler.current_process.as_ref().unwrap();
    let mut src_page_dir = Page::new(process.lock().pgdir.unwrap() as *mut u8);

    unsafe { copy_process_virtual_memory(&mut src_page_dir, &mut kernel_pgdir) };
    unsafe { compare_virtual_memory(&mut src_page_dir, &mut kernel_pgdir) };

    new_process.lock().mem_size = process.lock().mem_size;
    new_process.lock().parent = Some(Arc::clone(&process));
    new_process.lock().name = process.lock().name.clone();
    new_process.lock().state = ProcessState::READY;

    // Copy trapframe
    let parent_trapframe = unsafe { *process.lock().trapframe.unwrap() };
    unsafe { *new_process.lock().trapframe.unwrap() = parent_trapframe };
    unsafe { (*new_process.lock().trapframe.unwrap()).eax = 0 }; // Return 0 on child process

    unsafe { scheduler.resume() };
}

pub unsafe fn copy_process_virtual_memory(src_page_dir: &mut Page, dst_page_dir: &mut Page) {
    let page_entries = PAGE_SIZE / core::mem::size_of::<usize>();
    let src_page_dir_data = src_page_dir.cast_to::<usize>();
    let dst_page_dir_data = dst_page_dir.cast_to::<usize>();

    for i in 0..page_entries {
        if i * PAGE_SIZE.pow(2) > KERNEL_BASE {
            break;
        }

        let src_page_dir_entry = src_page_dir_data[i];
        let src_page_dir_flags = PTE_FLAGS!(src_page_dir_entry);

        // Empty page directory entry
        if src_page_dir_entry & PTE_P == 0 {
            continue;
        }

        // Allocate page directory entry
        let mut new_page_dir = allocate_page().expect("[ERROR] Failed to allocate page");
        let new_page_dir_address = V2P!(new_page_dir.as_ptr() as usize);
        dst_page_dir_data[i] = new_page_dir_address | src_page_dir_flags;

        // Avoid calling walk_page_dir here due to lookup slowdown
        let dst_page_dir_entry = dst_page_dir_data[i];
        let dst_page_dir_address = P2V!(PTE_ADDRESS!(dst_page_dir_entry)) as *mut usize;
        let dst_page_table_data = from_raw_parts_mut(dst_page_dir_address, PAGE_SIZE / 4);

        let src_page_dir_entry = src_page_dir_data[i];
        let src_page_dir_address = P2V!(PTE_ADDRESS!(src_page_dir_entry)) as *mut usize;
        let src_page_table_data = from_raw_parts_mut(src_page_dir_address, PAGE_SIZE / 4);

        for j in 0..page_entries {
            // Empty page table entry
            if src_page_table_data[j] & PTE_P == 0 {
                continue;
            }

            let src_page_address = PTE_ADDRESS!(src_page_table_data[j]);
            let src_page_flags = PTE_FLAGS!(src_page_table_data[j]);
            let mut new_page = allocate_page().expect("[ERROR] Failed to allocate page");
            let new_page_address = V2P!(new_page.as_ptr() as usize);
            dst_page_table_data[j] = new_page_address | src_page_flags;

            mem_move(
                P2V!(src_page_address) as *mut u8,
                P2V!(new_page_address) as *mut u8,
                PAGE_SIZE,
            );
        }
    }
}

pub fn wait() {
    let scheduler = unsafe { SCHEDULER.lock() };
    let process_list = unsafe { PROCESS_LIST.lock() };
    let parent = unsafe { scheduler.current_process.as_ref().unwrap() };

    loop {
        let mut has_children = false;

        // Check if any children are still alive
        for process in process_list.list.iter() {
            if process.lock().parent.is_none() {
                continue;
            }

            let parent_pid = process.lock().parent.as_ref().unwrap().lock().pid;
            if parent_pid != parent.lock().pid {
                continue;
            }

            if process.lock().state == ProcessState::KILLED {
                continue;
            }

            has_children = true;
        }

        if has_children == false {
            return;
        }

        sleep(parent.as_ref() as *const SpinMutex<Process> as usize);
    }
}
