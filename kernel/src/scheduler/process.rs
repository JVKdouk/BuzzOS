use core::panic;

use alloc::{string::String, sync::Arc, vec::Vec};

use super::{
    defs::process::{Context, Process, ProcessList, ProcessState, TrapFrame},
    scheduler::PROCESS_LIST,
};

use crate::{
    apic::mp::get_my_cpu,
    memory::{
        defs::{
            Page, KERNEL_BASE, KERNEL_DATA_SEGMENT, PAGE_SIZE, PTE_U, PTE_W, TASK_STATE_SEGMENT,
            USER_CODE_SEGMENT, USER_DATA_SEGMENT,
        },
        mem::{memmove, memset},
        vm::{allocate_page, map_pages, setup_kernel_page_tables},
    },
    sync::{
        cpu_cli::{pop_cli, push_cli},
        spin_mutex::{SpinMutex, SpinMutexGuard},
    },
    x86::{
        defs::PrivilegeLevel,
        helpers::{load_cr3, ltr},
    },
    V2P,
};

impl ProcessList {
    pub const fn new() -> Self {
        ProcessList(Vec::new())
    }

    pub fn get_pid(&self, pid: usize) -> Option<Arc<SpinMutex<Process>>> {
        if pid > self.0.len() {
            panic!("[FATAL] PID out of bounds");
        }

        match self.0.iter().find(|process| process.lock().pid == pid) {
            Some(process) => Some(Arc::clone(process)),
            None => None,
        }
    }

    /// Allocation of process requires an empty slot. Finds the next empty slot
    /// to allocate the process.
    pub fn insert_process(&mut self, mut process: Process) -> Option<usize> {
        match self
            .0
            .iter()
            .position(|p| p.lock().state == ProcessState::EMPTY)
        {
            Some(pid) => {
                process.pid = pid;
                self.0[pid] = Arc::new(SpinMutex::new(process));
                Some(pid)
            }
            None => {
                process.pid = self.0.len();
                self.0.push(Arc::new(SpinMutex::new(process)));
                Some(self.0.len() - 1)
            }
        }
    }

    pub fn get_next_ready(&self) -> Option<&Arc<SpinMutex<Process>>> {
        self.0
            .iter()
            .find(|p| p.lock().state == ProcessState::READY)
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
    let mut process = Process::new(0);
    let trapframe_size = core::mem::size_of::<TrapFrame>() as isize;
    let context_size = core::mem::size_of::<Context>() as isize;
    let kernel_page = allocate_page().address as *mut usize;
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
    match PROCESS_LIST.lock().insert_process(process) {
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
pub unsafe fn setup_user_virtual_memory(page_dir: Page, address: *const usize, size: usize) {
    if size >= PAGE_SIZE {
        panic!("[FATAL] User Virtual Memory is bigger than one page");
    }

    let memory_page = allocate_page();
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
pub unsafe fn spawn_init_process() {
    let process_pid = spawn_process().expect("[FATAL] Failed to start init process");
    let mut process = PROCESS_LIST.lock().get_pid(process_pid).unwrap();
    let mut process_lock = process.lock();

    let kernel_pgdir = setup_kernel_page_tables().expect("[FATAL] Failed to setup Kernel pages");

    let user_code_selector = (USER_CODE_SEGMENT << 3) as u16 | PrivilegeLevel::Ring3 as u16;
    let user_data_selector = (USER_DATA_SEGMENT << 3) as u16 | PrivilegeLevel::Ring3 as u16;

    process_lock.pgdir = Some(kernel_pgdir.address as *mut usize);

    setup_user_virtual_memory(
        kernel_pgdir,
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
    process_lock.name = String::from("init");
    process_lock.state = ProcessState::READY;
}
