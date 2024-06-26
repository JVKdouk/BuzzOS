use alloc::sync::Arc;

use crate::{
    apic::mp::get_my_cpu,
    memory::defs::KERNEL_BASE,
    memory::vm::KERNEL_PAGE_DIR,
    println,
    scheduler::process::switch_user_virtual_memory,
    sync::spin_mutex::SpinMutex,
    x86::helpers::{load_cr3, sti},
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
        let process = self.current_process.as_ref().unwrap();
        let process_lock = process.lock();
        let process_trapframe = process_lock.trapframe.unwrap();

        unsafe {
            *process_trapframe = *trapframe;
        }
    }

    pub fn get_trapframe(&self) -> Option<*mut TrapFrame> {
        match self.current_process.is_none() {
            true => None,
            false => {
                let process = self.current_process.as_ref().unwrap();
                let process_lock = process.lock();
                process_lock.trapframe.clone()
            }
        }
    }

    pub unsafe fn resume(&mut self) {
        let process = self.current_process.as_ref().unwrap();
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

        let cpu = get_my_cpu();
        let enable_interrupts = cpu.get_interrupt_state();

        switch(process_context, self.context);
        cpu.set_interrupt_state(enable_interrupts);
    }

    pub unsafe fn run_scheduler(&mut self) {
        SCHEDULER.force_unlock();

        loop {
            sti();

            let mut process_list = PROCESS_LIST.lock();

            // No need to unlock here, lock is dropped automatically
            let Some(process) = process_list.get_next_ready() else {
                PROCESS_LIST.force_unlock();
                core::hint::spin_loop(); // Hint to the processor that it should save power here
                continue;
            };

            // println!("RUNNING {}", process.lock().name);

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

            // Check for leftover CLI stack. If there is a leftover CLI stack, look for
            // a mutex lock that has been locked but never unlocked. At this point in execution
            // there should be no leftover CLI stack.
            let cpu = get_my_cpu();
            let number_cli = cpu.get_cli();
            if number_cli > 0 {
                panic!(
                    "[ERROR] Leftover CLI stack has been found for CPU {}",
                    cpu.apic_id
                );
            }
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
