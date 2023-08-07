use core::slice::from_raw_parts;

use alloc::string::{String, ToString};

use crate::{
    filesystem::fs::{find_inode_by_path, setup_file_system},
    interrupts::defs::system_call as SystemCall,
    println,
    scheduler::{
        defs::process::{ProcessState, TrapFrame},
        exec::exec,
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
        SCHEDULER.lock().set_trapframe(trapframe);
    }

    let system_call_number = trapframe.eax;
    let arg0 = trapframe.edi;
    let arg1 = trapframe.edx;
    let arg2 = trapframe.ecx;

    match system_call_number {
        SystemCall::PRINT_TRAP_FRAME => print_trapframe(),
        SystemCall::EXIT => exit(),
        SystemCall::YIELD => _yield(),
        SystemCall::SETUP_FS => setup_file_system(),
        SystemCall::EXEC => {
            let str_slice = unsafe { from_raw_parts(arg0 as *const u8, arg1) };
            let path = core::str::from_utf8(str_slice).unwrap();
            let inode = find_inode_by_path(path.to_string()).unwrap();
            println!("{} {}", system_call_number, path);
            exec(&inode).unwrap();
        }
        _ => panic_undefined_syscall(),
    };
}

pub fn print_trapframe() {
    let scheduler = unsafe { SCHEDULER.lock() };
    let trapframe = unsafe { *scheduler.get_trapframe().unwrap() };
    println!("{:#?}", trapframe);
}

pub fn _yield() {
    unsafe { SCHEDULER.lock().resume() };
}

pub fn exit() {
    unsafe {
        let mut scheduler = SCHEDULER.lock();
        scheduler.current_process.as_mut().unwrap().lock().state = ProcessState::KILLED;
        scheduler.resume();
    }
}
