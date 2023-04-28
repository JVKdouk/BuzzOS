pub const BASE_IRQ: usize = 32;

pub const IRQ_TIMER: usize = 0;
pub const IRQ_KEYBOARD: usize = 1;
pub const IRQ_COM1: usize = 4;
pub const IRQ_IDE: usize = 14;
pub const IRQ_ERROR: usize = 19;
pub const IRQ_SPURIOUS: usize = 31;

/// Local APIC Registers

pub mod local_apic_registers {
    pub const ID: usize = 0x20 / 4;
    pub const VERSION: usize = 0x30 / 4;
    pub const TASK_PRIORITY: usize = 0x80 / 4;
    pub const EOI: usize = 0xB0 / 4;
    pub const SPURIOUS_INTERRUPT: usize = 0xF0 / 4;
    pub const UNIT_ENABLE: usize = 0x100;
    pub const ERROR_STATUS: usize = 0x280 / 4;
    pub const INTERRUPT_COMMAND_LOW: usize = 0x300 / 4;
    pub const INTERRUPT_COMMAND_HIGH: usize = 0x310 / 4;
    pub const INIT: usize = 0x500;
    pub const STARTUP: usize = 0x600;
    pub const DELIVERY_STATUS: usize = 0x1000;
    pub const ASSERT_INTERRUPT: usize = 0x4000;
    pub const DEASSERT_INTERRUPT: usize = 0x0000;
    pub const LEVEL: usize = 0x8000;
    pub const BROADCAST: usize = 0x80000;
    pub const BUSY: usize = 0x1000;
    pub const FIXED: usize = 0x0;
    pub const TIMER: usize = 0x320 / 4;
    pub const TIMER_X1: usize = 0xB;
    pub const TIMER_PERIODIC: usize = 0x20000;
    pub const PERFORMANCE_COUNTER: usize = 0x340 / 4;
    pub const LOCAL_VECTOR_TABLE_0: usize = 0x350 / 4;
    pub const LOCAL_VECTOR_TABLE_1: usize = 0x360 / 4;
    pub const LOCAL_VECTOR_TABLE_3: usize = 0x370 / 4;
    pub const MASKED_INTERRUPT: usize = 0x10000;
    pub const TIMER_INITIAL_COUNT: usize = 0x380 / 4;
    pub const TIMER_CURRENT_COUNT: usize = 0x390 / 4;
    pub const TIMER_DIVIDE_CONFIGURATION: usize = 0x3E0 / 4;
}
