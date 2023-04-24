use core::{marker::PhantomData, mem::size_of};

use lazy_static::lazy_static;

use crate::{interrupts::interrupt_handlers::*, println, x86::helpers::lidt};

use super::defs::*;

extern "x86-interrupt" {
    fn trap_enter(stack: InterruptStackFrame);
}

impl<F> Gate<F> {
    // Implementation of an empty gate. Used to initialized gates
    #[inline]
    pub const fn empty() -> Self {
        // Ensure our gate is an interrupt at startup.
        let flags = GateFlags::INTGATE as u8;

        Gate {
            fn_addr_low: 0,
            fn_addr_high: 0,
            segment_selector: 0,
            reserved: 0,
            handler: PhantomData,
            flags,
        }
    }

    // Implementation of an empty gate. Used to initialized gates
    #[inline]
    pub const fn user_interrupt() -> Self {
        // Ensure our gate is an interrupt at startup.
        let flags = GateFlags::INTGATE as u8 | GateFlags::DPL3 as u8;

        Gate {
            fn_addr_low: 0,
            fn_addr_high: 0,
            segment_selector: 0,
            reserved: 0,
            handler: PhantomData,
            flags,
        }
    }

    pub const fn set_flags(&mut self, flags: u8) {
        self.flags = flags;
    }

    // Set gate handler. Accepts the 64-bits address of the handler function
    #[inline]
    pub unsafe fn set_handler_addr(&mut self, addr: u32) -> &mut u8 {
        self.fn_addr_low = addr as u16;
        self.fn_addr_high = (addr >> 16) as u16;
        self.segment_selector = 0x8;
        self.flags |= GateFlags::PRESENT as u8;
        &mut self.flags
    }
}

impl Gate<InterruptHandler> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: InterruptHandler) {
        let handler = handler as u32;
        unsafe { self.set_handler_addr(handler) };
    }
}

impl Gate<InterruptHandlerWithErr> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: InterruptHandlerWithErr) {
        let handler = handler as u32;
        unsafe { self.set_handler_addr(handler) };
    }
}

impl Gate<PageFaultHandler> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: PageFaultHandler) {
        let handler = handler as u32;
        unsafe { self.set_handler_addr(handler) };
    }
}

impl IDT {
    // Initialization of our Interrupt Descriptor Table. Reserved gates must also be initialized.
    // Notice gp_interrupts are also intiialized, being composed of 224 elements. Those are
    // interrupts available for the OS (e.g. System Calls).
    #[inline]
    pub fn new() -> IDT {
        IDT {
            div_by_zero: Gate::empty(),
            debug: Gate::empty(),
            non_maskable_interrupt: Gate::empty(),
            breakpoint: Gate::empty(),
            overflow: Gate::empty(),
            bound_range_exceeded: Gate::empty(),
            invalid_opcode: Gate::empty(),
            device_not_available: Gate::empty(),
            double_fault: Gate::empty(),
            coprocessor_segment_overrun: Gate::empty(),
            invalid_tss: Gate::empty(),
            segment_not_present: Gate::empty(),
            stack_segment_fault: Gate::empty(),
            gen_protection_fault: Gate::empty(),
            page_fault: Gate::empty(),
            reserved_1: Gate::empty(),
            x87_floating_point: Gate::empty(),
            alignment_check: Gate::empty(),
            machine_check: Gate::empty(),
            simd_floating_point: Gate::empty(),
            virtualization: Gate::empty(),
            control_protection_exception: Gate::empty(),
            reserved_2: [Gate::empty(); 6],
            hv_injection_exception: Gate::empty(),
            vmm_communication_exception: Gate::empty(),
            security_exception: Gate::empty(),
            reserved_3: Gate::empty(),
            gp_interrupts: [Gate::empty(); 256 - 32],
        }
    }

    /// Creates the descriptor pointer for this table. This pointer can only be
    /// safely used if the table is never modified or destroyed while in use.
    fn pointer(&self) -> InterruptDescriptorTablePointer {
        InterruptDescriptorTablePointer {
            base: self as *const _ as u32,
            limit: (size_of::<Self>() - 1) as u16,
        }
    }

    // This two-step load is necessary to ensure our IDT is available whenever
    // the CPU needs it. Notice a non-static reference would cause all sorts
    // of bugs related to free before use.
    #[inline]
    pub fn load(&'static self) {
        lidt(&self.pointer());
    }
}

lazy_static! {
    static ref GLOBAL_IDT: IDT = {
        let mut global_idt = IDT::new();

        // Setup User System Call Interrupt Handler
        unsafe {
            global_idt.gp_interrupts[32].set_flags(GateFlags::TRAPGATE as u8 | GateFlags::DPL3 as u8);
            global_idt.gp_interrupts[32].set_handler_addr(trap_enter as *const () as u32);
        }

        // Setup Handler
        global_idt.div_by_zero.set_handler_fn(div_by_zero_handler);
        global_idt.breakpoint.set_handler_fn(breakpoint_handler);
        global_idt.gen_protection_fault.set_handler_fn(gen_protection_fault);
        global_idt.double_fault.set_handler_fn(double_fault_handler);
        global_idt.page_fault.set_handler_fn(page_fault);
        global_idt.overflow.set_handler_fn(overflow);
        global_idt.bound_range_exceeded.set_handler_fn(bound_range);
        global_idt
    };
}

pub fn setup_idt() {
    GLOBAL_IDT.load();
    println!("[KERNEL] Interrupt Table Initialized");
}
