use core::slice::from_raw_parts;

use alloc::string::ToString;

use crate::{
    filesystem::fs::{find_inode_by_path, get_path_filename, setup_file_system},
    interrupts::defs::system_call as SystemCall,
    println,
    scheduler::{
        defs::process::{Process, ProcessState, TrapFrame},
        exec::exec,
        process::{fork, wait},
        scheduler::SCHEDULER,
        sleep::wakeup,
    },
    sync::spin_mutex::SpinMutex,
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
            let name = get_path_filename(path.to_string());
            exec(&inode, name);
        }
        SystemCall::FORK => fork(),
        SystemCall::WAIT => wait(),
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
    let mut scheduler = unsafe { SCHEDULER.lock() };
    let mut process = scheduler.current_process.as_mut().unwrap().lock();

    if process.parent.is_some() {
        let parent_process = process.parent.as_mut().unwrap().as_ref() as *const SpinMutex<Process>;
        wakeup(parent_process as usize);
    }

    unsafe {
        process.state = ProcessState::KILLED;
        SCHEDULER.lock().resume();
    }
}
