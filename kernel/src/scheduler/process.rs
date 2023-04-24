use core::panic;

use alloc::{string::String, vec::Vec};

use super::{
    defs::process::{Context, Process, ProcessList, ProcessState, TrapFrame},
    scheduler::PROCESS_LIST,
};

use crate::{
    interrupts::defs::InterruptStackFrame,
    memory::{
        defs::{
            Page, KERNEL_BASE, KERNEL_DATA_SEG_ENTRY, PAGE_SIZE, PTE_U, PTE_W, USER_CODE_SEG_ENTRY,
            USER_DATA_SEG_ENTRY,
        },
        gdt::TSS,
        mem::{memmove, memset},
        vm::{allocate_page, map_pages, setup_kernel_page_tables},
    },
    println,
    x86::{defs::PrivilegeLevel, helpers::load_cr3},
    V2P,
};

impl ProcessList {
    pub const fn new() -> Self {
        ProcessList(Vec::new())
    }

    pub fn get_pid(&self, pid: usize) -> Option<Process> {
        if pid > self.0.len() {
            panic!("[FATAL] PID out of bounds");
        }

        self.0.get(pid).cloned()
    }

    pub fn set_pid(&mut self, pid: usize, process: Process) {
        if pid > self.0.len() {
            panic!("[FATAL] PID out of bounds");
        }

        self.0[pid] = process;
    }

    pub fn kill_pid(&mut self, pid: usize) {
        if pid > self.0.len() {
            panic!("[FATAL] PID out of bounds");
        }

        self.0[pid].state = ProcessState::EMPTY;
    }

    /// Allocation of process requires an empty slot. Finds the next empty slot
    /// to allocate the process.
    pub fn set_empty_slot(&mut self, mut process: Process) -> Option<usize> {
        match self.0.iter().position(|p| p.state == ProcessState::EMPTY) {
            Some(pid) => {
                process.pid = pid;
                self.0[pid] = process;
                Some(pid)
            }
            None => {
                process.pid = self.0.len();
                self.0.push(process);
                Some(self.0.len() - 1)
            }
        }
    }

    pub fn get_next_ready(&self) -> Option<Process> {
        self.0
            .iter()
            .find(|p| p.state == ProcessState::READY)
            .cloned()
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
            pid,
        }
    }

    pub fn set_trapframe(&mut self, trapframe: TrapFrame) {
        unsafe {
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
    pub fn trap_enter(frame: InterruptStackFrame);
}

/// Spawn a process block. Notice the process block has no meaning until it is queued to be run
/// by the Scheduler. At this point, it is generally prepared to run in the user-space, but no
/// specific are provided.
pub unsafe fn spawn_process() -> Result<usize, &'static str> {
    let mut process = Process::new(0);
    let trapframe_size = core::mem::size_of::<TrapFrame>() as isize;
    let context_size = core::mem::size_of::<Context>() as isize;
    let kernel_page = allocate_page()?.address as *mut usize;
    let mut esp = kernel_page.offset(PAGE_SIZE as isize / 4);

    process.kernel_stack = Some(kernel_page);
    process.mem_size = PAGE_SIZE;
    process.state = ProcessState::EMBRYO;

    // Setup Trapframe Layout
    esp = esp.offset(-trapframe_size / 4);
    memset(esp as usize, 0, trapframe_size as usize);
    process.trapframe = Some(esp as *mut TrapFrame);

    // Setup Context Layout
    esp = esp.offset(-context_size / 4);
    memset(esp as usize, 0, context_size as usize);
    process.context = Some(esp as *mut Context);

    // Create Trap Return
    (*process.context.unwrap()).eip = trap_return as *const () as usize;

    // Return the pid for the newly created process slot
    match PROCESS_LIST.lock().set_empty_slot(process) {
        Some(pid) => Ok(pid),
        None => Err("Could not allocate"),
    }
}

/// Switch to User Virtual Memory, departing from Kernel Virtual Memory. Notice this does
/// not yet change the DPL or CPL, as this is done at a later step. Since Kernel memory is
/// entirely mapped for every process, the process can continue executing after the memory is
/// switched.
pub unsafe fn switch_user_virtual_memory(process: &Process) {
    let page_dir = process
        .pgdir
        .expect("[FATAL] Process has no page directory") as usize;

    let mut tss = TSS.lock();
    (*tss).esp0 = process.kernel_stack.unwrap().offset(PAGE_SIZE as isize) as u32;
    (*tss).ss0 = (KERNEL_DATA_SEG_ENTRY << 3) as u16;

    load_cr3(V2P!(page_dir));
}

/// Migrate from Kernel Virtual Memory to User Virtual Memory. Notice that
pub unsafe fn setup_user_virtual_memory(page_dir: Page, address: *const usize, size: usize) {
    if size >= PAGE_SIZE {
        panic!("[FATAL] User Virtual Memory is bigger than one page");
    }

    let memory_page = allocate_page().expect("[FATAL] Failed to Allocate");
    let virtual_address = 0;
    let page_size = PAGE_SIZE;
    let phys_address = V2P!(memory_page.address as usize);
    let flags = PTE_W | PTE_U;

    // Prepare all pages required by the process and copy over the executable binary and data
    memset(memory_page.address as usize, 0, PAGE_SIZE);
    map_pages(page_dir, virtual_address, page_size, phys_address, flags);
    memmove(address as usize, memory_page.address as usize, size);
}

/// Spawns the first process to run in the user space, the init process. Subsequent children inherit
/// many attributes of the init process, such as trapframe,
pub unsafe fn spawn_init_process() -> Result<(), &'static str> {
    let process_pid = spawn_process()?;
    let mut process = PROCESS_LIST.lock().get_pid(process_pid).unwrap();

    let kernel_pgdir = setup_kernel_page_tables()?;
    let user_code_selector = (USER_CODE_SEG_ENTRY << 3) as u16 | PrivilegeLevel::Ring3 as u16;
    let user_data_selector = (USER_DATA_SEG_ENTRY << 3) as u16 | PrivilegeLevel::Ring3 as u16;

    process.pgdir = Some(kernel_pgdir.address as *mut usize);

    setup_user_virtual_memory(
        kernel_pgdir,
        &_binary_init_start as *const usize,
        &_binary_init_size as *const usize as usize,
    );

    // Setup Trapframe
    (*process.trapframe.unwrap()).esp = PAGE_SIZE;
    (*process.trapframe.unwrap()).eip = 0;
    (*process.trapframe.unwrap()).cs = user_code_selector;
    (*process.trapframe.unwrap()).ds = user_data_selector;
    (*process.trapframe.unwrap()).es = user_data_selector;
    (*process.trapframe.unwrap()).ss = user_data_selector;
    (*process.trapframe.unwrap()).eflags = 0x0;

    // Setup Misc
    process.name = String::from("init");
    process.state = ProcessState::READY;

    PROCESS_LIST.lock().set_pid(process_pid, process);

    Ok(())
}
