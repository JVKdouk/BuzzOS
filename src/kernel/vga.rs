use crate::kernel::x86;
use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt::Write;

/* ************ Macros ************ */

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! clear {
    () => ($crate::kernel::vga::_clear());
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/* ************ Declarations ************ */

// Address to write to VGA Buffer
const VGA_COL_LIMIT: u32 = 80;
const VGA_ROW_LIMIT: u32 = 25;
const VGA_CHAR_LIMIT: u32 = VGA_COL_LIMIT * VGA_ROW_LIMIT;

// VGA Colors type definitions. 4 bits for color 4 bits for foreground
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VGAColor(u8);

// Color Pallete. Facilitates translating VGA text colors troughout the Kernel
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xA,
    LightCyan = 0xB,
    LightRed = 0xC,
    Pink = 0xD,
    Yellow = 0xE,
    White = 0xF,
}

// Represents a single char in the screen (16 bits long)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct VGAChar {
    byte: u8, // ASCII Characters only
    color: VGAColor,
}

// 2D array representing all chars in the screen
#[repr(transparent)] // Same memory ordering as its single field
struct Buffer {
    chars: [Volatile<VGAChar>; (VGA_ROW_LIMIT * VGA_COL_LIMIT) as usize],
}

pub struct VGAText {
    position: u32,
    color: VGAColor, // Current color of the VGA Writer
    buffer: &'static mut Buffer, // ' tells Rust this reference is valid forever (Valid for VGA)
}

/* ************ Definitions ************ */

// Implementation of VGA Color. New is invoked to create the packaged 8-bits color
impl VGAColor {
    // This function can be called to setup the Color background and foreground
    // If background is not passed, defaults to black
    pub const fn new(fg: Color, bg: Color) -> VGAColor {
        VGAColor((bg as u8) << 4 | (fg as u8))
    }
}

impl fmt::Write for VGAText {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        return Ok(());
    }
}

impl VGAText {
    pub unsafe fn init(&mut self, fg: Color, bg: Color) {
        // Disable blinking pointer
        x86::outb(0x3D4, 0x0A);
        x86::outb(0x3D5, 0x20);

        // Init foreground and background color
        self.set_color(fg, bg);
    }

    // Set foreground and background color
    pub unsafe fn set_color(&mut self, fg: Color, bg: Color) {
        let color: VGAColor = VGAColor::new(fg, bg);
        self.color = color;
    }
    
    // Write a single char to the display. In fact, writes 16 bits (half word) to the
    // VGA buffer. If character is a new line (\n), invokes new line handler
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            // Support new line character
            b'\n' => self.new_line(),

            // Any other ASCII character
            byte => {
                // If we go above limit (position-wise), scroll one line up
                if self.position >= VGA_CHAR_LIMIT {
                    self.scroll(-1);
                }

                let color = self.color;
                self.buffer.chars[self.position as usize].write(VGAChar { byte, color });
                self.position += 1;
            }
        }
    }

    // Writes a full string to the VGA buffer. Iterates through all characters and print one by
    // one. If character is not ASCII, prints ASCII diamond "◆"
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // If the character matches ASCII table, move on
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // If not, print the diamond character
                _ => self.write_byte(0x4),
            }
        }
    }

    // Clear VGA buffer by setting every char back to 0
    pub fn clear(&mut self) -> fmt::Result {
        for i in 0..(VGA_CHAR_LIMIT) {
            self.buffer.chars[i as usize].write(VGAChar { byte: 0x0, color: self.color });
        }

        self.position = 0;

        return Ok(());
    }

    // Implements scrolling functionality to VGA buffer. Negative values scroll up, Positive values
    // scroll down.
    fn scroll(&mut self, offset: i32) -> fmt::Result {
        let index = VGA_COL_LIMIT as i32 * offset;
        
        // Update the position of every character on the screen
        for i in 0..(self.buffer.chars.len()) {
            let calculated_offset = (i as i32) + index;

            // If i is past the position pointer, we can stop early
            if (i > self.position as usize) {
                break;
            }

            // Values before the scroll offset should be skipped
            if calculated_offset < 0 {
                continue;
            } else {
                let move_vga: VGAChar = self.buffer.chars[i].read();
                self.buffer.chars[calculated_offset as usize].update(|current| current.byte = move_vga.byte);
            }

            // If this is the case, nothing after the position pointer matters. Zero it
            if (i > (VGA_CHAR_LIMIT - VGA_COL_LIMIT) as usize) {
                self.buffer.chars[i].update(|current| current.byte = 0x0);
            }
        }

        // Update text pointer position. If position would be negative, bound it back to zero
        if (self.position as i32 - index >= 0) {
            self.position = self.position.wrapping_add_signed(index);
        } else {
            self.position = 0;
        }

        return Ok(());
    }

    // New line handler, calculates next line and jump to it
    fn new_line(&mut self) {
        let next_line: u32 = (self.position as u32 / VGA_COL_LIMIT) + 1;
        
        // self.buffer.chars[self.position as usize].write(VGAChar { byte: '\n' as u8, color: self.color });
        self.position = (next_line * VGA_COL_LIMIT) as u32;
    }
}

// Static instance of our Writer. It is lazy_static because Rust initializes
// statics at compile time. We could have const, but Rust cannot resolve pointer
// references at compile time, so we need to use lazy_static and tell Rust
// to only initialize this variable at run time.
lazy_static! {
    pub static ref TEXT: Mutex<VGAText> = Mutex::new(VGAText {
        position: 0,
        color: VGAColor::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// Writer print helper. Simple procedure of locking Writer, and write print!() macro
// arguments
pub fn _print(args: fmt::Arguments) {
    TEXT.lock().write_fmt(args).unwrap();
}

// Writer clear buffer helper. Allows the usage of the macro clear!()
pub fn _clear() {
    TEXT.lock().clear().unwrap();
}