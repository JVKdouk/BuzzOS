#![no_std]
#![no_main]
#![allow(unused)]
#![feature(mixed_integer_ops)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#[macro_use]

pub mod devices;
pub mod interrupts;
pub mod memory;
pub mod misc;
pub mod threading;
pub mod x86;

// Interface definition of panic in Rust. Core represents the core library
use core::panic::PanicInfo;

// Uses C calling convention instead of Rust. no_mangle removes name mangling when compiled.
// _start is the default entry point for most systems. Function is diverging as the Kernel should
// never return
#[no_mangle]
pub unsafe extern "C" fn _start(mem_layout_pointer: usize) -> ! {
    /// Initialize debugging method (VGA or Console)
    devices::debug::debug_init();
    devices::debug::_clear();
    misc::logo::print_logo();

    /// Setup Segmentation and Virtual Memory
    memory::gdt::setup_gdt();
    memory::vm::setup_vm(mem_layout_pointer);

    /// Setup Interrupts
    interrupts::idt::setup_idt();

    x86::helpers::int3();
    loop {}
}

// Once the Kernel panics, enter an infinite loop
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print!("{}", _info);
    loop {}
}
