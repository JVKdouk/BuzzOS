use core::sync::atomic::Ordering;

use crate::{
    apic::mp::{get_my_cpu, IS_CPU_MAPPED},
    x86::helpers::{cli, read_eflags, sti},
};

pub fn push_cli() {
    if IS_CPU_MAPPED.load(Ordering::Relaxed) {
        // Clear interrupts as soon as possible
        let eflags = read_eflags();
        cli();

        let cpu = get_my_cpu();
        let number_cli = cpu.get_cli();
        let interrupt_status = ((eflags >> 9) & 0x1) > 0;

        // Save the current state of the Interrupt Flag
        if number_cli == 0 {
            cpu.set_interrupt_state(interrupt_status);
        }

        // Add to the cli calling stack
        cpu.set_cli(number_cli + 1);
    }
}

pub fn pop_cli() {
    if IS_CPU_MAPPED.load(Ordering::Relaxed) {
        let cpu = get_my_cpu();
        let mut number_cli = cpu.get_cli();
        let enable_interrupts = cpu.get_interrupt_state();

        if number_cli >= 1 {
            number_cli -= 1;
            cpu.set_cli(number_cli);
        }

        if number_cli == 0 && enable_interrupts == true {
            sti();
        }
    }
}
