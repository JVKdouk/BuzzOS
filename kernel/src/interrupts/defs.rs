use bitflags::bitflags;
use core::marker::PhantomData;

/// System Call Constants (system_call.rs)

pub mod system_call {
    pub const PRINT_TRAP_FRAME: usize = 0;
    pub const EXIT: usize = 1;
    pub const YIELD: usize = 2;
}

/// Structure of a pointer to a IDT. Must be passed in this format
/// to a lidt call.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed(2))]
pub struct InterruptDescriptorTablePointer {
    pub limit: u16, // Size
    pub base: u32,  // Pointer to Starting Address
}

/// The generic parameter defines what handler should be used (with/without error description,
/// page fault, etc).
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Gate<F> {
    pub fn_addr_low: u16,        // Function Address Low (16 bits)
    pub segment_selector: u16,   // Segment Selector (16 bits)
    pub reserved: u8,            // Reserver by Processor (8 bits)
    pub flags: u8,               // Flags (16 bits)
    pub fn_addr_high: u16,       // Function Address High (32 bits)
    pub handler: PhantomData<F>, // Phanthom Handler. This field does not exist in the final struct
}

/// Default definition of the IDT. First 32 interrupts are used by the processor to communicate
/// exceptions (such as page fault, divide by zero, etc). After the initial 32 gates, the next 224
/// entries can be used by the OS to throw custom interrupts/traps.
#[repr(C)]
pub struct IDT {
    /// Definition of Processor Exceptions.
    /// Complete list can be found here: https://wiki.osdev.org/Exceptions
    pub div_by_zero: Gate<InterruptHandler>,
    pub debug: Gate<InterruptHandler>,
    pub non_maskable_interrupt: Gate<InterruptHandler>,
    pub breakpoint: Gate<InterruptHandler>,
    pub overflow: Gate<InterruptHandler>,
    pub bound_range_exceeded: Gate<InterruptHandler>,
    pub invalid_opcode: Gate<InterruptHandler>,
    pub device_not_available: Gate<InterruptHandler>,
    pub double_fault: Gate<InterruptHandlerWithErr>,

    /// Deprecated: Segment Overruns are handle by the GPF now.
    /// We need to include it to fit all exceptions in the IDT correctly.
    pub coprocessor_segment_overrun: Gate<InterruptHandler>,

    pub invalid_tss: Gate<InterruptHandlerWithErr>,
    pub segment_not_present: Gate<InterruptHandlerWithErr>,
    pub stack_segment_fault: Gate<InterruptHandler>,
    pub gen_protection_fault: Gate<InterruptHandlerWithErr>,
    pub page_fault: Gate<PageFaultHandler>,

    pub reserved_1: Gate<InterruptHandler>,

    pub x87_floating_point: Gate<InterruptHandler>,
    pub alignment_check: Gate<InterruptHandlerWithErr>,
    pub machine_check: Gate<InterruptHandler>,
    pub simd_floating_point: Gate<InterruptHandler>,
    pub virtualization: Gate<InterruptHandler>,
    pub control_protection_exception: Gate<InterruptHandlerWithErr>,

    pub reserved_2: [Gate<InterruptHandler>; 6],

    pub hv_injection_exception: Gate<InterruptHandler>,
    pub vmm_communication_exception: Gate<InterruptHandlerWithErr>,

    pub security_exception: Gate<InterruptHandlerWithErr>,
    pub reserved_3: Gate<InterruptHandler>,

    /// Those can be defined by the OS (Notice 0 to 31 are already used by the processor)
    pub gp_interrupts: [Gate<InterruptHandler>; 256 - 32],
}

/// Gate Flags. Those allow fine grain control of how and when should traps/interrupts be issued.
/// You can see more at https://wiki.osdev.org/Interrupt_Descriptor_Table#Gate_Descriptor_2
pub enum GateFlags {
    INTGATE = 0b1110,   // Is it an interrupt?
    TRAPGATE = 0b1111,  // Is it a trap?
    DISABLE = 0b1,      // Is interrupts enabled?
    DPL0 = 0b00 << 5,   // Permission Level of 0 (Kernel)
    DPL1 = 0b01 << 5,   // Permission Level of 1
    DPL2 = 0b10 << 5,   // Permission Level of 2
    DPL3 = 0b11 << 5,   // Permission Level of 3 (User)
    PRESENT = 0b1 << 7, // Is it present?
}

/// Once an interrupt occurs, the CPU saves old stack pointers (rsp and ss), align the stack pointer,
/// switch user-level to kernel-level stacks, update RFLAGS register (status bits), saves the instruction
/// pointer, saves the error code, and finally invokes the interrupt handler (controlled by the IDT).
/// More information can be found here: https://wiki.osdev.org/Interrupt
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InterruptStack {
    value: InterruptStackFrame,
}

/// This is the stack frame for when an interrupt is issued. It follows the x86-interrupt calling convention.
/// It is different from normal stack frames, and are used to store all details regarding the current
/// running instruction before the interrupt handler is called.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    /// Where to return after interrupt is completed
    pub instruction_pointer: u32,
    /// The code segment selector, padded with zeros.
    pub code_segment: u32,
    /// The flags register before the interrupt handler was invoked.
    pub cpu_flags: u32,
    /// The stack pointer at the time of the interrupt.
    pub stack_pointer: u32,
    /// The stack segment descriptor at the time of the interrupt (often zero in 64-bit mode).
    pub stack_segment: u32,
}

/// Type of an interrupt handler without error. Notice the calling convention "x86-interrupt." This tells
/// Rust how to behave when it comes to handling these functions. Function calls are done voluntarily,
/// on the other hand, interrupts are not. Registers cannot be voluntarily saved when an interrupt occurs,
/// since we don't know, at compile time, if an instruction will produce an exception. To go around this
/// limitation, x86-interrupt calling convention forces all registers to be backed up and later restored.
/// Some handlers have error codes, others do not.
pub type InterruptHandler = extern "x86-interrupt" fn(InterruptStackFrame);
pub type InterruptHandlerWithErr = extern "x86-interrupt" fn(InterruptStackFrame, error_code: u32);
pub type PageFaultHandler =
    extern "x86-interrupt" fn(InterruptStackFrame, error_code: PageFaultErr);

// Page Faults have more information regarding how and where it happened. Must be handled
// different than other gates.
bitflags! {
    #[repr(transparent)]
    pub struct PageFaultErr: u32 {
        const FAILURE_TYPE = 1;          // Failure Type (0 = Not Present; 1 = Protected)
        const WRITE_FAILURE = 1 << 1;    // Operation Type (0 = Read; 1 = Write)
        const CPL_USER = 1 << 2;         // Exception Address-Space (0 = Kernel; 1 = User).
        const PTE_MALFORMATION = 1 << 3; // PTE Structure (1 = Malformed; 0 = Normal)
        const INST_FETCH = 1 << 4;       // Fetch Type (0 = Data; 1 = Instruction)
        const PROTECTION_KEY = 1 << 5;   // Protection Key Failure (0 = Normal; 1 = Failure)
        const SS = 1 << 6;               // Shadow Stack Violation (0 = Normal; 1 = Violated)
        const SGX = 1 << 15;             // Memory Enclave Violation (0 = Normal; 1 = Violated)
    }
}
