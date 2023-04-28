use crate::{apic::local_apic::local_apic_acknowledge, devices::console::CONSOLE};

use super::defs::InterruptStackFrame;

pub extern "x86-interrupt" fn timer(_frame: InterruptStackFrame) {
    local_apic_acknowledge();
}

pub extern "x86-interrupt" fn keyboard(_frame: InterruptStackFrame) {
    CONSOLE.lock().keyboard_interrupt();
    local_apic_acknowledge();
}
