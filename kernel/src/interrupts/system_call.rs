use crate::{
    interrupts::defs::system_call as SystemCall,
    println,
    scheduler::{
        defs::process::{ProcessState, TrapFrame},
        scheduler::SCHEDULER,
    },
};

/// If a call to an undefined System Call happens, panic and exit.
fn panic_undefined_syscall() {
    println!("[WARNING] Invalid system call");
    exit();
}

/// Every System Call passes through this handler. The trapframe is passed to facilitate loading
/// the ABI registers and getting the system call number in eax.
pub fn handle_system_call(trapframe: &mut TrapFrame) {
    unsafe {
        // Update trapframe, which contains all registers of the process execution context
        SCHEDULER
            .lock()
            .current_process
            .as_mut()
            .unwrap()
            .set_trapframe(*trapframe);
    }

    let system_call_number = trapframe.eax;
    let arg0 = trapframe.edi;
    let arg1 = trapframe.esi;
    let arg2 = trapframe.edx;
    let arg3 = trapframe.ecx;

    match system_call_number {
        SystemCall::PRINT_TRAP_FRAME => print_trapframe(),
        SystemCall::EXIT => exit(),
        SystemCall::YIELD => _yield(),
        _ => panic_undefined_syscall(),
    };
}

pub fn print_trapframe() {
    let scheduler = unsafe { SCHEDULER.lock() };
    let current_process = scheduler.current_process.as_ref().unwrap();
    let trapframe = current_process.get_trapframe().unwrap();
    println!("{:#?}", trapframe);
}

pub fn _yield() {
    unsafe { SCHEDULER.lock().resume() };
}

pub fn exit() {
    unsafe {
        let mut scheduler = SCHEDULER.lock();
        scheduler.current_process.as_mut().unwrap().state = ProcessState::KILLED;
        scheduler.resume();
    }
}
