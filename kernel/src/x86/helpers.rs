use core::arch::asm;

use crate::{
    println,
    {
        interrupts::defs::InterruptDescriptorTablePointer,
        memory::defs::GlobalDescriptorTablePointer,
    },
};

use super::defs::*;

// ******** Control Registers ********

#[inline]
pub fn lcr3(page_dir: usize) {
    unsafe {
        asm!("mov cr3, {}", in(reg) page_dir, options(nostack, preserves_flags));
    }
}

// ******** Interrupts ********

#[inline]
pub fn lidt(idt: &InterruptDescriptorTablePointer) {
    unsafe {
        asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
    }
}

#[inline]
pub fn cli() {
    unsafe {
        asm!("cli");
    }
}

#[inline]
pub fn sti() {
    unsafe {
        asm!("sti");
    }
}

// *************** Segmentation ***************

#[inline]
pub unsafe fn lgdt(gdt: &GlobalDescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]",
        in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
}

#[inline]
pub fn load_cs(sel: u16) {
    unsafe {
        asm!("pushl {0:e}; \
        pushl $1f; \
        lretl; \
        1:", in(reg) sel as u32, options(att_syntax));
    }
}

#[inline]
pub fn set_gs(v: u16) {
    unsafe {
        asm!("gs {0:x}", in(reg) v, options(readonly, nostack, preserves_flags));
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

#[inline]
pub fn stosb(address: usize, value: u8, length: usize) {
    unsafe {
        asm!(
            "cld; \
            rep stosb;",
            in("al") value,
            in("edi") address,
            in("ecx") length,
            options(nostack, preserves_flags)
        );
    }
}

/// Cause a breakpoint exception by invoking the `int3` instruction.
#[inline]
pub fn int3() {
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}
