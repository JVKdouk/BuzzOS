use crate::{scheduler::defs::process::ProcessState, x86::helpers::cli};

use super::scheduler::{PROCESS_LIST, SCHEDULER};

struct SleepLock {
    pid: usize,
    locked: bool,
    object: usize,
}

/// The idea of sleep is to remove the process from the ready/running process queue until
/// the object it is waiting for is ready. Sleep is accompanied by the wakeup method, which
/// together are capable of putting processes to sleep and then adding them back to the queue
/// once the wakeup signal is emitted.
pub fn sleep(object: usize) {
    let mut scheduler_lock = unsafe { SCHEDULER.lock() };

    let mut current_process_lock =
        unsafe { scheduler_lock.get_current_process() }.expect("[ERROR] Sleep on empty scheduler");
    let mut current_process = current_process_lock.lock();

    // Put process to sleep and release lock
    current_process.state = ProcessState::SLEEPING;
    current_process.sleep_object = object;
    unsafe { current_process_lock.force_unlock() };

    unsafe { scheduler_lock.resume() };

    // Ensure interrupts are clear until the execution returns to the process
    cli();

    // Clean up process from sleep and reacquire lock
    let mut current_process = current_process_lock.lock();
    current_process.sleep_object = 0;
}

/// Wakeup is a signal to all processes waiting on an object that the object they request is
/// ready, and therefore they should come back from sleep. This functions runs through the process
/// list and wakes up all processes that rely on the provided object.
pub fn wakeup(object: usize) {
    let mut process_list = unsafe { PROCESS_LIST.lock() };
    process_list.0.iter_mut().for_each(|process_lock| {
        let mut process = process_lock.lock();
        if process.sleep_object == object && process.state == ProcessState::SLEEPING {
            process.state = ProcessState::READY;
        }
    });
}
