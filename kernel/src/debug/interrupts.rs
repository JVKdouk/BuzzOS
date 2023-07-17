use crate::{apic::mp::get_my_cpu, println, x86::helpers::read_eflags};

pub fn decode_eflags() {
    let eflags = read_eflags();
    let if_enabled = (eflags >> 9) & 0x1;

    println!("\n--- EFlags ---");
    println!("EFlags: 0b{:b}", eflags);
    println!("Interrupt Enabled: {}", if_enabled > 0);
    println!("--- EFlags ---\n");
}
