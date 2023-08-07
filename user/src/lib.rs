#![no_std]
#![no_main]

use core::panic::PanicInfo;

pub mod libs;

pub fn issue_system_call(sys_call_number: usize) {
    unsafe {
        core::arch::asm!("int 64", in("eax") sys_call_number);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
