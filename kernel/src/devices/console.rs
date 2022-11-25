use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use super::uart::uart_put_char;

pub struct Console {
    panicked: bool,
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        return Ok(());
    }
}

impl Console {
    fn write_char(&self, c: char) {
        uart_put_char(c);
    }

    pub fn write_string(&self, text: &str) {
        for c in text.chars() {
            self.write_char(c);
        }
    }
}

lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console { panicked: false });
}
