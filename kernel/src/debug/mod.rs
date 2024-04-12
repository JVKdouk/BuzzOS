use crate::{apic::mp::get_my_cpu, println, scheduler::scheduler::PROCESS_LIST};

pub mod interrupts;
pub mod process;
pub mod vm;

pub fn debug_cpu() {
    let cpu = get_my_cpu();

    let apic_id = cpu.apic_id;
    let number_cli = cpu.get_cli();
    let enable_interrupt = cpu.get_interrupt_state();

    println!("\n--- CPU ({apic_id}) ---");
    println!("Number CLI: {}", number_cli);
    println!("Interrupt Enabled? {}", enable_interrupt);
    println!("--- CPU ({apic_id}) ---\n");
}

pub fn debug_process_list() {
    let process_list = unsafe { PROCESS_LIST.lock() };
    println!("{:#?}", process_list.list);
}
