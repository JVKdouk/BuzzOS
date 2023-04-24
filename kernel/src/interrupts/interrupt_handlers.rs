use core::arch::asm;

use crate::{
    println,
    scheduler::{defs::process::TrapFrame, scheduler::SCHEDULER},
    x86::helpers::read_cr2,
};

use super::{
    defs::{InterruptStackFrame, PageFaultErr},
    system_call::handle_system_call,
};

pub extern "x86-interrupt" fn div_by_zero_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVISION BY ZERO\n{:#?}", frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, _error_code: PageFaultErr) {
    panic!(
        "[FATAL] Page Fault - eip: 0x{:X} - cr2: 0x{:X}",
        frame.instruction_pointer,
        read_cr2()
    );
}

pub extern "x86-interrupt" fn non_maskable(frame: InterruptStackFrame) {
    println!("EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn overflow(frame: InterruptStackFrame) {
    println!("EXCEPTION: OVERFLOW\n{:#?}", frame);
}

pub extern "x86-interrupt" fn bound_range(frame: InterruptStackFrame) {
    println!("EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", frame);
}

pub extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _err: u32) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#X?}", frame);
}

pub extern "x86-interrupt" fn gen_protection_fault(frame: InterruptStackFrame, _err: u32) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", frame);
}

#[no_mangle]
extern "C" fn user_interrupt_handler(trapframe: &mut TrapFrame) {
    unsafe {
        // Update trapframe, which contains all registers of the process execution context
        let current_process = SCHEDULER
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
    handle_system_call(system_call_number, arg0, arg1, arg2, arg3);
}
