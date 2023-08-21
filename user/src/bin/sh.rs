#![no_std]
#![no_main]

#[allow(unused_imports)]
use user::libs::*;

fn fib(count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    if count == 1 {
        return 1;
    }

    return fib(count - 1) + fib(count - 2);
}

#[no_mangle]
pub extern "C" fn _start(argc: usize, argv: *const usize) {
    unsafe { core::arch::asm!("mov eax, {}", in(reg) fib(30)) }
}
