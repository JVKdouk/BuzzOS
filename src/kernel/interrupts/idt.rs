use core::{marker::PhantomData, mem::size_of};

use lazy_static::lazy_static;

use crate::{
    kernel::x86::{
        defs::{Segment, SegmentSelector, CS},
        helpers::lidt,
    },
    println,
};

use super::defs::*;

impl<F> Gate<F> {
    // Implementation of an empty gate. Used to initialized gates
    #[inline]
    pub const fn empty() -> Self {
        // Ensure our gate is an interrupt at startup.
        let flags = GateFlags::INTGATE as u16;

        Gate {
            fn_addr_low: 0,
            fn_addr_middle: 0,
            fn_addr_high: 0,
            segment_selector: 0,
            reserved: 0,
            handler: PhantomData,
            flags,
        }
    }

    // Set gate handler. Accepts the 64-bits address of the handler function
    #[inline]
    pub unsafe fn set_handler_addr(&mut self, addr: u64) -> &mut u16 {
        self.fn_addr_low = addr as u16;
        self.fn_addr_middle = (addr >> 16) as u16;
        self.fn_addr_high = (addr >> 32) as u32;
        self.segment_selector = CS::get_reg().0;
        self.flags |= GateFlags::PRESENT as u16;
        println!(
            "{:X} {:X} {:X} {:X} {:X}",
            self.fn_addr_low,
            self.fn_addr_middle,
            self.fn_addr_high,
            self.segment_selector,
            self.flags
        );
        &mut self.flags
    }
}

impl Gate<InterruptHandler> {
    #[inline]
    pub fn set_handler_fn(&mut self, handler: InterruptHandler) {
        let handler = handler as u64;
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
            divide_by_zero: Gate::empty(),
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
            general_protection_fault: Gate::empty(),
            page_fault: Gate::empty(),
            reserved_1: Gate::empty(),
            x87_floating_point: Gate::empty(),
            alignment_check: Gate::empty(),
            machine_check: Gate::empty(),
            simd_floating_point: Gate::empty(),
            virtualization: Gate::empty(),
            cp_protection_exception: Gate::empty(),
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
            base: self as *const _ as u64,
            limit: (size_of::<Self>() - 1) as u16,
        }
    }

    // This two-step load is necessary to ensure our IDT is available whenever
    // the CPU needs it. Notice a non-static reference would cause all sorts
    // of bugs related to free before use.
    #[inline]
    pub fn load(&'static self) {
        unsafe { self.load_unsafe() }
    }

    #[inline]
    pub unsafe fn load_unsafe(&self) {
        unsafe {
            lidt(&self.pointer());
        }
    }
}

lazy_static! {
    static ref GLOBAL_IDT: IDT = {
        let mut global_idt = IDT::new();
        global_idt.breakpoint.set_handler_fn(breakpoint_handler);
        global_idt
    };
}

pub fn setup_idt() {
    GLOBAL_IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStack) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
