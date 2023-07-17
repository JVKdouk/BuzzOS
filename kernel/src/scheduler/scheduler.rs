use alloc::sync::Arc;

use crate::{
    apic::mp::get_my_cpu,
    debug::{debug_cpu, debug_process_list, interrupts::decode_eflags},
    memory::defs::KERNEL_BASE,
    memory::vm::KERNEL_PAGE_DIR,
    println,
    scheduler::{process::switch_user_virtual_memory, sleep::wakeup},
    sync::spin_mutex::SpinMutex,
    x86::helpers::{hlt, load_cr3, sti},
    V2P,
};

use super::defs::{
    process::{Context, Process, ProcessList, ProcessState, TrapFrame},
    scheduler::{Scheduler, SchedulerState},
};

pub static mut PROCESS_LIST: SpinMutex<ProcessList> = SpinMutex::new(ProcessList::new());
pub static mut SCHEDULER: SpinMutex<Scheduler> = SpinMutex::new(Scheduler::new());

extern "C" {
    fn switch(scheduler_context: *mut *mut Context, process_context: *mut Context);
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            current_process: None,
            context: 0x0 as *mut Context, // TODO: Improve this
            status: SchedulerState::READY,
        }
    }

    pub fn get_current_process(&self) -> Option<Arc<SpinMutex<Process>>> {
        match self.current_process.as_ref() {
            None => None,
            Some(process) => Some(Arc::clone(process)),
        }
    }

    pub fn set_trapframe(&mut self, trapframe: *mut TrapFrame) {
        let mut process = self.current_process.as_ref().unwrap();
        let mut process_lock = process.lock();
        let mut process_trapframe = process_lock.trapframe.unwrap();

        unsafe {
            *process_trapframe = *trapframe;
        }
    }

    pub fn get_trapframe(&self) -> Option<*mut TrapFrame> {
        match self.current_process.is_none() {
            true => None,
            false => {
                let mut process = self.current_process.as_ref().unwrap();
                let mut process_lock = process.lock();
                process_lock.trapframe.clone()
            }
        }
    }

    pub unsafe fn resume(&mut self) {
        let mut process = self.current_process.as_ref().unwrap();
        let mut process_lock = process.lock();

        // Handle current process state
        process_lock.state = if process_lock.state == ProcessState::RUNNING {
            ProcessState::READY
        } else {
            process_lock.state
        };

        // Prepare process context for switching
        let process_context = process_lock.context.as_mut().unwrap();

        SCHEDULER.force_unlock();
        PROCESS_LIST.force_unlock();
        process.force_unlock();

        let cpu = get_my_cpu().unwrap();
        let mut enable_interrupts = unsafe { &mut *cpu.enable_interrupt.get() };
        let interrupt_status = *enable_interrupts;
        switch(process_context, self.context);
        *enable_interrupts = interrupt_status;
    }

    pub unsafe fn run_scheduler(&mut self) {
        SCHEDULER.force_unlock();

        loop {
            let mut process_list = PROCESS_LIST.lock();

            // No need to unlock here, lock is dropped automatically
            let Some(mut process) = process_list.get_next_ready() else {
                PROCESS_LIST.force_unlock();
                core::hint::spin_loop(); // Hint to the processor that it should save power here
                continue;
            };

            let mut process_lock = process.lock();
            let process_context = process_lock.context.expect("[FATAL] No Context");
            process_lock.state = ProcessState::RUNNING;

            // Update Scheduler and Process States
            self.status = SchedulerState::BUSY;
            self.current_process = Some(Arc::clone(process));

            PROCESS_LIST.force_unlock();
            process.force_unlock();

            unsafe {
                switch_user_virtual_memory(process);
                switch(&mut self.context, process_context);
                switch_kernel_virtual_memory();
            };

            self.current_process = None;
            self.status = SchedulerState::READY;

            sti();
        }
    }
}

pub unsafe fn switch_kernel_virtual_memory() {
    load_cr3(V2P!(KERNEL_PAGE_DIR.lock().unwrap()));
}

pub fn setup_scheduler() {
    unsafe {
        SCHEDULER.lock().run_scheduler();
    }
}
