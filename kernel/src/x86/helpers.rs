use core::arch::asm;

use crate::{
    println,
    {
        interrupts::defs::InterruptDescriptorTablePointer,
        memory::defs::GlobalDescriptorTablePointer,
    },
};

use super::defs::*;

// *************** Segmentation ***************

impl Segment for CS {
    fn get_reg() -> u16 {
        let segment: u16;
        unsafe {
            asm!("mov {0:x}, cs", out(reg) segment, options(nomem, nostack, preserves_flags));
        }
        segment
    }

    unsafe fn set_reg(sel: u16) {
        unsafe {
            asm!(
                "push {sel}",
                "lea {tmp}, [1f + rip]",
                "push {tmp}",
                "retfq",
                "1:",
                sel = in(reg) u64::from(sel),
                tmp = lateout(reg) _,
                options(preserves_flags),
            );
        }
    }
}

// ******** Interrupt Description Table ********

#[inline]
pub unsafe fn lidt(idt: &InterruptDescriptorTablePointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }
}

// *************** GDT ***************

#[inline]
pub unsafe fn lgdt(gdt: &GlobalDescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
}

// ************ I/O Ports ************

#[inline]
pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline]
pub unsafe fn outw(port: u16, value: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline]
pub fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

#[inline]
pub unsafe fn inw(port: u16) -> u32 {
    let value: u32;
    unsafe {
        asm!(
            "in eax, dx",
            out("eax") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}

/// Cause a breakpoint exception by invoking the `int3` instruction.
#[inline]
pub fn int3() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}
