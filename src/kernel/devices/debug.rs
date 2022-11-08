use core::fmt;
use core::fmt::Write;

use super::console::CONSOLE;
use super::uart;
use super::vga::{self, TEXT};

/* ************ Macros ************ */

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel::devices::debug::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! clear {
    () => {
        $crate::kernel::devices::debug::_clear()
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn debug_init() {
    #[cfg(feature = "vga")]
    vga::TEXT.lock().init(vga::Color::White, vga::Color::Black);

    #[cfg(feature = "console")]
    uart::uart_init();
}

// Switches between printing methods
pub fn _print(args: fmt::Arguments) {
    #[cfg(feature = "vga")]
    TEXT.lock().write_fmt(args).unwrap();

    #[cfg(feature = "console")]
    CONSOLE.lock().write_fmt(args).unwrap();
}

// Switches between clearing methods
pub fn _clear() {
    #[cfg(feature = "vga")]
    TEXT.lock().clear().unwrap();
}
