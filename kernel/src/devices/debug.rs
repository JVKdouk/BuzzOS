use core::fmt;
use core::fmt::Write;

use super::console::CONSOLE;
use super::uart;

/* ************ Macros ************ */

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::devices::debug::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn debug_init() {
    uart::uart_init().expect("[ERR] Failed to Setup UART");
}

// Switches between printing methods
pub fn _print(args: fmt::Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
}
