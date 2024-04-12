use crate::{
    apic::local_apic::local_apic_acknowledge,
    interrupts::system_calls::exit,
    memory::{
        defs::{Page, PTE_U},
        vm::walk_page_dir,
    },
    println,
    scheduler::{defs::process::TrapFrame, scheduler::SCHEDULER},
    x86::helpers::read_cr2,
};

use super::{
    defs::{InterruptStackFrame, PageFaultErr},
    irqs::handle_irq,
    system_calls::handle_system_call,
};

pub extern "x86-interrupt" fn div_by_zero_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: DIVISION BY ZERO\n{:#?}", frame);
}

pub extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, _error_code: PageFaultErr) {
    let address = read_cr2();

    let scheduler = unsafe { SCHEDULER.lock() };

    if scheduler.current_process.is_none() {
        panic!(
            "[FATAL] Kernel produced a page fault\nEIP: 0x{:X}\nCR2: 0x{:X}\n",
            frame.instruction_pointer, address
        );
    }

    let process = scheduler.current_process.as_ref().unwrap();
    let page_dir_ptr = process.lock().pgdir.unwrap();
    let mut page_dir = Page::new(page_dir_ptr as *mut u8);
    let page_entry = walk_page_dir(&mut page_dir, address, false);

    // Process hits the return trap. It has finished execution and should be killed.
    if address == 0xFFFFFFFF {
        println!("[WARNING] Return Trap - {}", process.lock().name);
        exit();
    }

    // Stack overflow happens when a write is performend on the guard page
    if page_entry.is_ok() && unsafe { *page_entry.unwrap() & PTE_U == 0 } {
        println!("[WARNING] Stack Overflow - {}", process.lock().name);
        unsafe { SCHEDULER.force_unlock() };
        exit();
    }

    // If a page fault occurs while running the Kernel, we need to panic
    if scheduler.current_process.is_none() {
        panic!(
            "[ERROR] Kernel Page Fault\nEIP: 0x{:X}\nCR2: 0x{:X}",
            frame.instruction_pointer, address
        )
    }

    println!(
        "\n[WARNING] Page Fault\nProcess Name: {}\nEIP: 0x{:X}\nCR2: 0x{:X}\n",
        process.lock().name,
        frame.instruction_pointer,
        address
    );

    exit();
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

pub extern "x86-interrupt" fn invalid_tss(frame: InterruptStackFrame, _err: u32) {
    println!(
        "EXCEPTION: INVALID TSS {:#X?}, Error Code: {:X}\n",
        frame, _err
    );
}

pub extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _err: u32) {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#X?}", frame);
}

pub extern "x86-interrupt" fn gen_protection_fault(frame: InterruptStackFrame, _err: u32) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}", frame);
}

pub extern "x86-interrupt" fn general_irq_handler(_frame: InterruptStackFrame) {
    println!("IRQ");
    local_apic_acknowledge();
}

#[no_mangle]
extern "C" fn interrupt_manager(trapframe: &mut TrapFrame) -> isize {
    // If Trap Number is 64, then this is a System Call, and not an IRQ
    if trapframe.trap_number == 64 {
        let output = handle_system_call(trapframe);
        return output.unwrap_or(0x0) as isize;
    }

    handle_irq(trapframe);
    0x0
}
