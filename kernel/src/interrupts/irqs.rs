use crate::{
    apic::{
        defs::{IRQ_COM1, IRQ_IDE, IRQ_KEYBOARD, IRQ_TIMER},
        local_apic::local_apic_acknowledge,
    },
    devices::console::CONSOLE,
    filesystem::ide::interrupt_ide,
    scheduler::{defs::process::TrapFrame, scheduler::SCHEDULER},
};

use super::system_call::_yield;

pub fn handle_irq(trapframe: &mut TrapFrame) {
    let irq_number = trapframe.trap_number - 32;

    match irq_number {
        IRQ_TIMER => timer(trapframe),
        IRQ_COM1 => keyboard(trapframe),
        IRQ_KEYBOARD => keyboard(trapframe),
        IRQ_IDE => interrupt_ide(),
        _ => local_apic_acknowledge(),
    }
}

fn timer(_trapframe: &mut TrapFrame) {
    // Do Something Here
    local_apic_acknowledge();

    // Clock Tick: Yield
    let is_running_process = unsafe { SCHEDULER.lock().current_process.is_some() };
    if is_running_process {
        _yield();
    }
}

fn keyboard(_trapframe: &mut TrapFrame) {
    // println!("HERE");
    local_apic_acknowledge();
    CONSOLE.lock().keyboard_interrupt();
}
