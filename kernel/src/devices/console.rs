use core::{fmt, sync::atomic::Ordering};
use lazy_static::lazy_static;

use crate::{
    apic::{
        defs::{IRQ_COM1, IRQ_KEYBOARD},
        io_apic::enable_irq,
    },
    sync::spin_mutex::SpinMutex,
};

use super::uart::{uart_get_char, uart_put_char, IS_UART_ENABLED};

pub struct Console;

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        return Ok(());
    }
}

impl Console {
    fn write_char(&self, c: char) {
        print_char_strategy_manager(c);
    }

    pub fn write_string(&self, text: &str) {
        // Serial safety check
        if IS_UART_ENABLED.load(Ordering::Relaxed) == false {
            panic!("[FATAL] UART is not open");
        }

        for c in text.chars() {
            self.write_char(c);
        }
    }

    pub fn keyboard_interrupt(&self) {
        while let Some(data) = uart_get_char() {
            self.write_char(data as char);
        }
    }
}

lazy_static! {
    pub static ref CONSOLE: SpinMutex<Console> = SpinMutex::new(Console {});
}

pub fn print_char_strategy_manager(c: char) {
    match c as u8 {
        // Backspace
        127 => {
            uart_put_char(0x8 as char);
            uart_put_char(' ');
            uart_put_char(0x8 as char);
        }

        // Carriage Return becomes Line Feed
        13 => uart_put_char('\n'),

        // All other characters
        _ => uart_put_char(c),
    }
}

pub fn setup_console() {
    enable_irq(IRQ_KEYBOARD, 0);
    enable_irq(IRQ_COM1, 0);
}
