use core::arch::asm;
use lazy_static::lazy_static;

use crate::{
    interrupts::defs::system_call::*,
    println,
    scheduler::{process::TrapFrame, scheduler::SCHEDULER},
};

/// If a call to an undefined System Call happens, panic and exit.
/// TODO: Change this to an exit of the process instead of killing the system.
fn panic_undefined_syscall() {
    panic!("[FATAL] Undefined System Call");
}

lazy_static! {
    /// Add your own System Calls here. Notice parameters use the System V ABI calling convention,
    /// so edi, esi, edx, and ecx registers are used to pass the first four parameters to parameters.
    /// Functions are passed as addresses in order to avoid Rust parameter validation.
    static ref SYSTEM_CALLS: [usize; NUM_SYS_CALLS] = {
        let panic_handler_address = panic_undefined_syscall as *const () as usize;
        let mut sys_calls = [panic_handler_address; NUM_SYS_CALLS];

        sys_calls[PRINT_TRAPFRAME_SYSCALL] = print_trapframe as *const () as usize;

        sys_calls
    };
}

/// Every System Call passes through this handler. The trapframe is passed to facilitate loading
/// the ABI registers and getting the system call number in eax.
pub fn handle_system_call(trapframe: &TrapFrame) {
    let system_call_number = trapframe.eax;

    if system_call_number > NUM_SYS_CALLS - 1 {
        panic_undefined_syscall();
    }

    let system_call_fn = SYSTEM_CALLS[system_call_number];

    unsafe {
        asm!("
        pusha

        mov eax, {trapframe}

        mov edi, [eax]
        mov esi, [eax + 4]
        mov ecx, [eax + 24]
        mov edx, [eax + 20]

        mov eax, ebx
        call eax
        
        popa
        ",
            trapframe = in(reg) trapframe as *const TrapFrame as usize,
            in("ebx") system_call_fn
        );
    }
}

pub fn print_trapframe() {
    let trapframe = unsafe { SCHEDULER.lock().get_trapframe().unwrap() };
    println!("{:#?}", unsafe { (*trapframe).clone() });
}
