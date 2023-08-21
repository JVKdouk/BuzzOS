use crate::{apic::mp::get_my_cpu, println, scheduler::scheduler::PROCESS_LIST};

pub mod interrupts;
pub mod vm;

pub fn debug_cpu() {
    let cpu = get_my_cpu().unwrap();

    let apic_id = cpu.apic_id;
    let number_cli = unsafe { *cpu.number_cli.get() };
    let enable_interrupt = unsafe { *cpu.enable_interrupt.get() };

    println!("\n--- CPU ({apic_id}) ---");
    println!("Number CLI: {}", number_cli);
    println!("Interrupt Enabled? {}", enable_interrupt);
    println!("--- CPU ({apic_id}) ---\n");
}

pub fn debug_process_list() {
    let process_list = unsafe { PROCESS_LIST.lock() };
    println!("{:#?}", process_list.list);
}
