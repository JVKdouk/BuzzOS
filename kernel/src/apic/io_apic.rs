use crate::x86::helpers::cpuid;

use super::{defs::BASE_IRQ, mp::IO_APIC};

/// Advance Programmable Interrupt Controller is an upgrade to the older PIC. It is used in
/// Multiprocessor systems. See more: https://wiki.osdev.org/APIC

pub const CPUID_APIC_FLAG_BIT: usize = 0x200;

pub const IO_APIC_ID_REG: usize = 0x0;
pub const IO_APIC_VERSION_REG: usize = 0x1;
pub const IO_APIC_REDIRECTION_TABLE: usize = 0x10;

pub const IO_APIC_INTERRUPT_DISABLE: usize = 0x00010000;

pub struct IOApic {
    register: usize,
    _padding: [usize; 3],
    data: usize,
}

pub fn check_apic() {
    let (_, _, edx) = cpuid(1);
    let has_apic = edx & CPUID_APIC_FLAG_BIT > 0;

    if !has_apic {
        panic!("[FATAL] APIC unavailable");
    }
}

fn io_apic_write(register: usize, data: usize) {
    let io_apic = unsafe { IO_APIC.lock().unwrap().as_mut().unwrap() };
    io_apic.register = register;
    io_apic.data = data;
}

pub fn io_apic_read(register: usize) -> usize {
    let io_apic = unsafe { IO_APIC.lock().unwrap().as_mut().unwrap() };
    io_apic.register = register;
    io_apic.data
}

pub fn setup_io_apic() {
    let max_number_irqs = (io_apic_read(IO_APIC_VERSION_REG) >> 16) & 0xFF;

    // APIC Registers are 64 bit long but must be written in two sets of 32 bits
    // operations.
    for i in 0..=max_number_irqs {
        // Select interrupt handler. For now, all interrupts are disabled. They will
        // be enabled as needed by other components of the system.
        io_apic_write(
            IO_APIC_REDIRECTION_TABLE + 2 * i,
            IO_APIC_INTERRUPT_DISABLE | (BASE_IRQ + i),
        );

        // Make interrupt receivable by all CPUs
        io_apic_write(IO_APIC_REDIRECTION_TABLE + 2 * i + 1, 0);
    }
}

pub fn enable_irq(irq: usize, cpu: usize) {
    io_apic_write(IO_APIC_REDIRECTION_TABLE + 2 * irq, BASE_IRQ + irq);
    io_apic_write(IO_APIC_REDIRECTION_TABLE + 2 * irq + 1, cpu << 24);
}
