use spin::Mutex;

use crate::{
    memory::defs::KERNEL_BASE, memory::vm::KERNEL_PAGE_DIR, println,
    scheduler::process::switch_user_virtual_memory, x86::helpers::load_cr3, V2P,
};

use super::defs::{
    process::{Context, ProcessList, ProcessState, TrapFrame},
    scheduler::{Scheduler, SchedulerState},
};

pub static mut PROCESS_LIST: Mutex<ProcessList> = Mutex::new(ProcessList::new());
pub static mut SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

extern "C" {
    fn switch(scheduler_context: *mut *mut Context, process_context: *mut Context);
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            current_process: None,
            context: unsafe { 0x0 as *mut Context }, // TODO: Improve this
            status: SchedulerState::READY,
        }
    }

    pub fn set_trapframe(&mut self, trapframe: *mut TrapFrame) {
        let process = self.current_process.as_mut().unwrap();
        let process_trapframe = process.trapframe.unwrap();

        unsafe {
            *process_trapframe = *trapframe;
        }

        // match self.current_process.is_none() {
        //     true => return,
        //     false => self.current_process.as_mut().unwrap().trapframe = Some(trapframe),
        // }
    }

    pub fn get_trapframe(&self) -> Option<*mut TrapFrame> {
        match self.current_process.is_none() {
            true => return None,
            false => self.current_process.as_ref().unwrap().trapframe,
        }
    }

    pub unsafe fn resume(&mut self) {
        let mut process_list = PROCESS_LIST.lock();

        // Save current state of the process back to the process list
        let mut current_process = self.current_process.as_mut().unwrap();
        let is_killed = current_process.state == ProcessState::KILLED;
        current_process.state = if is_killed {
            ProcessState::EMPTY
        } else {
            ProcessState::READY
        };

        // TODO: Do we really need to clone on every switch?
        process_list.set_pid(current_process.pid, current_process.clone());

        // Prepare process context for switching
        let mut process_context = current_process.context.as_mut().unwrap();

        SCHEDULER.force_unlock();
        PROCESS_LIST.force_unlock();
        switch(process_context, self.context);
    }

    pub unsafe fn run_scheduler(&mut self) {
        SCHEDULER.force_unlock();

        loop {
            // No need to unlock here, lock is droped automatically
            let Some(mut process) = PROCESS_LIST.lock().get_next_ready() else {
                continue;
            };

            let process_context = process.context.expect("[FATAL] No Context");
            let trapframe = process.trapframe.expect("[FATAL] No Trapframe");

            // Update Scheduler and Process States
            process.state = ProcessState::RUNNING;
            self.status = SchedulerState::BUSY;
            self.current_process = Some(process);

            unsafe {
                switch_user_virtual_memory(self.current_process.as_ref().unwrap());
                switch(&mut self.context, process_context);
                switch_kernel_virtual_memory();
            };

            self.status = SchedulerState::READY;
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
