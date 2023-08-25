use crate::{apic::mp::get_my_cpu, println, x86::helpers::read_eflags};

pub fn decode_eflags() {
    let eflags = read_eflags();
    let if_enabled = (eflags >> 9) & 0x1;

    println!("\n--- EFlags ---");
    println!("EFlags: 0b{:b}", eflags);
    println!("Interrupt Enabled: {}", if_enabled > 0);
    println!("--- EFlags ---\n");
}

pub fn debug_cpu_interrupts() {
    let if_enabled = (read_eflags() >> 9) & 0x1;
    let cpu_interrupts = get_my_cpu().get_interrupt_state();
    let number_cli = get_my_cpu().get_cli();

    println!("\n--- CPU Interrupts ---");
    println!("EFlags Interrupt Enabled: {}", if_enabled > 0);
    println!("CPU Interrupt Enabled: {}", cpu_interrupts);
    println!("CPU Number CLI: {}", number_cli);
    println!("--- CPU Interrupts ---\n");
}

pub fn read_cpu_number_cli() {
    let number_cli = get_my_cpu().get_cli();

    if number_cli > 0 {
        panic!("[ERROR] Number CLI is {}", number_cli);
    } else {
        println!("[DEBUG] Number CLI is {}", number_cli);
    }
}
