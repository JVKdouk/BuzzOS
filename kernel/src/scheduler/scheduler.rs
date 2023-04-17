use spin::Mutex;

use crate::{
    scheduler::process::switch_user_virtual_memory, structures::heap_linked_list::HeapLinkedList,
};

use super::defs::{
    process::{Process, ProcessState, TrapFrame},
    scheduler::Scheduler,
};

pub static mut PROCESS_LIST: Mutex<HeapLinkedList<Process>> = Mutex::new(HeapLinkedList::new());
pub static mut SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

extern "C" {
    fn switch(scheduler_context: usize, process_context: usize);
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            current_process: None,
            context: 0,
        }
    }

    pub fn set_trapframe(&mut self, trapframe: *mut TrapFrame) {
        match self.current_process.is_none() {
            true => return,
            false => self.current_process.as_mut().unwrap().trapframe = Some(trapframe),
        }
    }

    pub fn get_trapframe(&self) -> Option<*mut TrapFrame> {
        match self.current_process.is_none() {
            true => return None,
            false => self.current_process.as_ref().unwrap().trapframe,
        }
    }

    pub fn schedule(&mut self) -> Option<()> {
        let mut process = unsafe { PROCESS_LIST.lock().pop()? };
        let mut process_context = process.context.expect("[FATAL] No Context");
        process.state = ProcessState::RUNNING;
        self.current_process = Some(process);

        unsafe {
            switch_user_virtual_memory(self.current_process.as_ref().unwrap());

            switch(
                &self.context as *const usize as usize,
                process_context as usize,
            );
        };

        Some(())
    }

    /// Main execution loop. If there is a process to schedule and this CPU is not currently
    /// busy running another process, takes the next process and schedule it.
    pub fn run(&mut self) {
        unsafe {
            // At this moment, SCHEDULER has been locked so any future access is prohibited.
            // At this point we can unlock it for future use.
            SCHEDULER.force_unlock();
        }

        loop {
            if self.current_process.is_none() {
                self.schedule();
            }
        }
    }
}

pub fn setup_scheduler() {
    unsafe {
        SCHEDULER.lock().run();
    }
}
