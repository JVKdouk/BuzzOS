#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]
#![feature(pointer_byte_offsets)]
#[macro_use]

pub mod devices;
pub mod apic;
pub mod filesystem;
pub mod interrupts;
pub mod memory;
pub mod misc;
pub mod scheduler;
pub mod structures;
pub mod threading;
pub mod x86;

extern crate alloc;

// Interface definition of panic in Rust. Core represents the core library
use core::panic::PanicInfo;

use crate::x86::helpers::{cli, sti};

// Uses C calling convention instead of Rust. no_mangle removes name mangling when compiled.
// _start is the default entry point for most systems. Function is diverging as the Kernel should
// never return
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    // Initialize debugging method (VGA or Console)
    devices::debug::debug_init();
    misc::logo::print_logo();

    // Setup Virtual Memory
    memory::vm::setup_vm();
    memory::heap::setup_heap();

    // Setup Hardware Interrupts and Multiprocessing
    apic::mp::setup_mp();
    apic::local_apic::setup_local_apic();
    apic::io_apic::setup_io_apic();
    apic::disable_pic();

    // Setup Segmentation and Virtual Memory
    memory::gdt::setup_gdt();

    // Setup Interrupts
    devices::console::setup_console();
    interrupts::idt::setup_idt();

    sti();

    // File System
    devices::pci::map_pci_buses();
    filesystem::ide::setup_ide();

    // Scheduler
    scheduler::process::spawn_init_process();
    scheduler::scheduler::setup_scheduler();

    // Should never proceeed
    panic!("[FATAL] Returned from Scheduler");
}

// Once the Kernel panics, enter an infinite loop
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    cli();
    print!("{}", _info);
    loop {}
}

#[alloc_error_handler]
fn alloc_panic(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
