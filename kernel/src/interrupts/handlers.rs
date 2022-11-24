use crate::println;

use super::defs::{InterruptStack, InterruptStackFrame};

pub extern "x86-interrupt" fn div_by_zero_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, err: u64) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", frame);
}
