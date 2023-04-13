use spin::Mutex;

use crate::{
    scheduler::process::{switch_user_virtual_memory, ProcessState},
    structures::heap_linked_list::HeapLinkedList,
};

use super::process::Process;

struct Scheduler {
    current_process: Option<Process>,
}

pub static mut PROCESS_LIST: Mutex<HeapLinkedList<Process>> = Mutex::new(HeapLinkedList::new());
static mut SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

extern "C" {
    fn switch(process_context: usize);
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            current_process: None,
        }
    }

    pub fn schedule(&mut self) -> Option<()> {
        let mut process = unsafe { PROCESS_LIST.lock().pop()? };
        let mut process_context = process.context.expect("[FATAL] No Context");
        process.state = ProcessState::RUNNING;
        self.current_process = Some(process);

        unsafe {
            switch_user_virtual_memory(self.current_process.as_ref().unwrap());
            switch(process_context as usize)
        };

        Some(())
    }

    /// Main execution loop. If there is a process to schedule and this CPU is not currently
    /// busy running another process, takes the next process and schedule it.
    pub fn run(&mut self) {
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
