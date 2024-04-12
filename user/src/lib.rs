#![no_std]
#![no_main]
#![feature(const_mut_refs)]

extern crate alloc;

use core::panic::PanicInfo;

// use alloc::boxed::Box;

pub mod libs;

pub fn issue_system_call(sys_call_number: usize) {
    // let a = Box::new(1);

    unsafe {
        core::arch::asm!("int 64", in("eax") sys_call_number);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
