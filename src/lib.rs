#![no_std]
#![no_main]
#![allow(unused)]
#![feature(mixed_integer_ops)]

#[macro_use]
mod kernel {
    pub mod misc;

    pub mod vga;
    pub use vga::TEXT as vga_writer;

    mod x86;
    pub use x86::outb;

    pub mod vm;

    pub mod defs;
}

// Interface definition of panic in Rust. Core represents the core library
use core::panic::PanicInfo;

// Uses C calling convention instead of Rust. no_mangle removes name mangling when compiled.
// _start is the default entry point for most systems. Function is diverging as the Kernel should
// never return
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Setup VGA Text Mode
    kernel::vga_writer.lock().init(kernel::vga::Color::White, kernel::vga::Color::Black);
    kernel::misc::logo();
    println!("[INIT] VGA Text Mode Enabled");
    
    // Setup Paging System and Virtual Memory
    println!("[INIT] Mapping Memory");
    kernel::vm::setup_vm();

    let a = 0;
    loop {}
}

// Once the Kernel panics, enter an infinite loop
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print!("{}", _info);
    loop {}
}
