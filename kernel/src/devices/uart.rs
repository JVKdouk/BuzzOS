use core::panic;

/// UART Serial Communication implementation. COM1 is an external device located at 0x3F8. Communication with it
/// allows us to setup UART configuration and send our first bit of data.
/// More information can be found here https://wiki.osdev.org/UART.
use lazy_static::lazy_static;
use spin::Mutex;

use crate::x86::helpers::{inb, outb};

use super::defs::COM1;

// Ensures safety when talking to UART
lazy_static! {
    pub static ref IS_UART_ENABLED: Mutex<bool> = Mutex::new(false);
}

/// Initialize UART and perform its configuration. In case UART is not avaialable, it returns an error.
pub fn uart_init() -> Result<(), ()> {
    outb(COM1 + 2, 0x00); // FIFO Control Register
    outb(COM1 + 3, 0x80); // Line Control (Unlock Divisor)
    outb(COM1 + 0, (115200 / 9600) as u8); // Data Buffer
    outb(COM1 + 1, 0x00); // Interrupt Disable
    outb(COM1 + 3, 0x03); // Line Control (Lock Divisor, 8 data bits)
    outb(COM1 + 4, 0x00); // Modem Control
    outb(COM1 + 1, 0x01); // Interrupt Enable

    // If Line Status = 0xFF, no Serial Port is available
    if inb(COM1 + 5) == 0xFF {
        return Err(());
    }

    *IS_UART_ENABLED.lock() = true;

    // Enable interrupts
    inb(COM1 + 2);
    inb(COM1 + 0);

    Ok(())
}

/// Puts a character in the Serial Port
pub fn uart_put_char(c: char) {
    // UART safety check
    if *IS_UART_ENABLED.lock() == false {
        panic!("[FATAL] UART is not open");
    }

    // Serial needs to be ready. Waits for status line to be ready before
    // sending a character.
    for _ in 0..128 {
        let ready = (inb(COM1 + 5) & 0x20) > 0;
        if ready {
            break;
        }
    }

    // Send char to UART
    outb(COM1 + 0, c as u8);
}

pub fn uart_get_char() -> Option<u8> {
    // if !(*IS_UART_ENABLED.lock()) {
    //     return None;
    // }

    let status = inb(COM1 + 5) & 0x1;
    if status == 0 {
        return None;
    }

    return Some(inb(COM1));
}
