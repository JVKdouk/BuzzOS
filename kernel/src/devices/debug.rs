use core::fmt;
use core::fmt::Write;

use super::console::CONSOLE;
use super::uart;

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

/// Interface for different output methods (VGA, UART, I2C, Network, etc).
/// This is where you need to edit to add support to other methods. Args is a fmt
/// list of arguments that need to be printted.
pub fn _print(args: fmt::Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
}
