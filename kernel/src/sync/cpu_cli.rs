use crate::{
    apic::mp::{get_my_cpu, IS_CPU_MAPPED},
    x86::helpers::{cli, read_eflags, sti},
};

pub fn push_cli() {
    if unsafe { IS_CPU_MAPPED } {
        // Clear interrupts as soon as possible
        let eflags = read_eflags();
        cli();

        let cpu = get_my_cpu().unwrap();
        let mut number_cli = unsafe { &mut *cpu.number_cli.get() };
        let interrupt_status = ((eflags >> 9) & 0x1) > 0;

        // Save the current state of the Interrupt Flag
        if *number_cli == 0 {
            let mut enable_interrupts = unsafe { &mut *cpu.enable_interrupt.get() };
            *enable_interrupts = interrupt_status;
        }

        // Add to the cli calling stack
        *number_cli += 1;
    }
}

pub fn pop_cli() {
    if unsafe { IS_CPU_MAPPED } {
        let mut cpu = get_my_cpu();
        let mut number_cli = unsafe { &mut *cpu.unwrap().number_cli.get() };
        let mut enable_interrupts = unsafe { &mut *cpu.unwrap().enable_interrupt.get() };

        if *number_cli >= 1 {
            *number_cli -= 1;
        }

        if *number_cli == 0 && *enable_interrupts == true {
            sti();
        }
    }
}
