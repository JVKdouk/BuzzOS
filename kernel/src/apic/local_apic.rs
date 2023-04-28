use crate::{apic::defs::local_apic_registers as regs, apic::mp::LOCAL_APIC};

use super::defs::{
    local_apic_registers::{MASKED_INTERRUPT, PERFORMANCE_COUNTER, VERSION},
    BASE_IRQ, IRQ_ERROR, IRQ_SPURIOUS, IRQ_TIMER,
};

pub fn setup_local_apic() {
    // Enable local APIC
    local_apic_write(
        regs::SPURIOUS_INTERRUPT,
        regs::UNIT_ENABLE | (BASE_IRQ + IRQ_SPURIOUS),
    );

    // Setup APIC timer. Countes down based on the bus frequency and then issues an interrupt.
    // If necessary, use an external tool to calibrate the timer so it issues an interrupt on a
    // given interval.
    local_apic_write(regs::TIMER_DIVIDE_CONFIGURATION, regs::TIMER_X1);
    local_apic_write(regs::TIMER, regs::TIMER_PERIODIC | (BASE_IRQ + IRQ_TIMER));
    local_apic_write(regs::TIMER_INITIAL_COUNT, 4000000000);

    // Disable logical interrupts
    local_apic_write(regs::LOCAL_VECTOR_TABLE_0, regs::MASKED_INTERRUPT);
    local_apic_write(regs::LOCAL_VECTOR_TABLE_1, regs::MASKED_INTERRUPT);

    // Disable performance counter overflow if supported
    if ((local_apic_read(VERSION) >> 16) & 0xFF) >= 4 {
        local_apic_write(PERFORMANCE_COUNTER, MASKED_INTERRUPT);
    }

    // Redirect error interrupts
    local_apic_write(regs::LOCAL_VECTOR_TABLE_3, BASE_IRQ + IRQ_ERROR);

    // Clear error status registers
    local_apic_write(regs::ERROR_STATUS, 0);
    local_apic_write(regs::ERROR_STATUS, 0);

    // Ackowledge previous interrupts
    local_apic_write(regs::EOI, 0);

    // Synchronise arbitration ID
    local_apic_write(regs::INTERRUPT_COMMAND_HIGH, 0);
    local_apic_write(
        regs::INTERRUPT_COMMAND_LOW,
        regs::BROADCAST | regs::INIT | regs::LEVEL,
    );

    while (local_apic_read(regs::INTERRUPT_COMMAND_LOW) & regs::DELIVERY_STATUS) > 0 {}

    // Enable Interrupts on the APIC
    local_apic_write(regs::TASK_PRIORITY, 0);
}

pub fn get_local_apic_id() -> usize {
    local_apic_read(regs::ID)
}

pub fn local_apic_acknowledge() {
    local_apic_write(regs::EOI, 0);
}

fn local_apic_read(register: usize) -> usize {
    let register_handle = unsafe { LOCAL_APIC.lock().unwrap() };
    unsafe { *register_handle.add(register) }
}

fn local_apic_write(register: usize, data: usize) {
    let register_handle = unsafe { LOCAL_APIC.lock().unwrap() };
    unsafe { *register_handle.add(register) = data }

    // We must wait for the write to finish. This is done by reading.
    unsafe { *register_handle.add(regs::ID) };
}
